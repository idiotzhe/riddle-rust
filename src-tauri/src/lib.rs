use axum::{
    routing::{get, post},
    Router,
    response::{IntoResponse, Response},
    body::Body,
    http::{header, StatusCode, Uri},
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use sqlx::sqlite::SqlitePoolOptions;
use tera::Tera;
use std::sync::Arc;
use socketioxide::{SocketIo, extract::SocketRef};
use rust_embed::RustEmbed;
use tauri::Manager;

mod db;
mod models;
mod handlers;
mod utils;

#[derive(RustEmbed)]
#[folder = "../template/"]
struct Asset;

pub struct AppState {
    db: sqlx::SqlitePool,
    tera: Tera,
    io: SocketIo,
}

async fn start_backend(db_path: std::path::PathBuf) {
    // 使用更加健壮的连接字符串，确保在 AppData 目录下寻找或创建 lantern.db
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
    println!(">>> Connecting to database: {}", db_url);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&db_url)
        .await
        .expect("Failed to connect to database. Ensure you have write permissions.");

    // 显式设置 SQLite 的繁忙超时
    let _ = sqlx::query("PRAGMA busy_timeout = 5000;").execute(&pool).await;

    // --- 自动初始化数据库表结构 ---
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL,
            avatar TEXT,
            user_code TEXT UNIQUE,
            token TEXT UNIQUE,
            register_time DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE IF NOT EXISTS riddles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            question TEXT NOT NULL,
            remark TEXT,
            options_json TEXT NOT NULL DEFAULT '[]',
            answer TEXT NOT NULL,
            add_time DATETIME DEFAULT CURRENT_TIMESTAMP,
            is_solved BOOLEAN DEFAULT 0,
            solver_id INTEGER,
            FOREIGN KEY (solver_id) REFERENCES users(id)
        );
        CREATE TABLE IF NOT EXISTS activities (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            start_time DATETIME NOT NULL,
            end_time DATETIME NOT NULL
        );
        CREATE TABLE IF NOT EXISTS guess_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            riddle_id INTEGER NOT NULL,
            solve_time DATETIME DEFAULT CURRENT_TIMESTAMP,
            is_solved BOOLEAN DEFAULT 0,
            FOREIGN KEY (user_id) REFERENCES users(id),
            FOREIGN KEY (riddle_id) REFERENCES riddles(id),
            UNIQUE(user_id, riddle_id)
        );
    "#).execute(&pool).await.expect("Failed to initialize database tables");

    // 初始化 Tera 实例
    let mut tera = if cfg!(debug_assertions) {
        // 开发模式：从磁盘加载
        Tera::new("template/**/*.html").expect("Failed to load templates from disk")
    } else {
        // 生产模式：使用嵌入资源
        let mut t = Tera::default();
        let mut templates = Vec::new();
        for file in Asset::iter() {
            if file.ends_with(".html") {
                if let Some(content) = Asset::get(&file) {
                    let template_str = std::str::from_utf8(content.data.as_ref()).expect("UTF-8 error");
                    templates.push((file.to_string(), template_str.to_string()));
                }
            }
        }
        t.add_raw_templates(templates).expect("Failed to load embedded templates");
        t
    };

    // 【关键修复】确保过滤器在两种模式下都注册
    tera.register_filter("get_time_range_display", move |value: &serde_json::Value, _args: &std::collections::HashMap<String, serde_json::Value>| {
        if value.is_null() {
            return Ok(serde_json::Value::String("".to_string()));
        }

        let start_str = value.get("start_time").and_then(|v| v.as_str()).unwrap_or("");
        let end_str = value.get("end_time").and_then(|v| v.as_str()).unwrap_or("");
        
        if start_str.is_empty() || end_str.is_empty() {
            return Ok(serde_json::Value::String("".to_string()));
        }

        // 尝试多种常见的日期格式进行解析
        let parse_dt = |s: &str| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") // 带 T
                .or_else(|_| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")) // 不带 T
                .or_else(|_| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S.%f")) // 带毫秒
                .ok()
        };

        let start = parse_dt(start_str);
        let end = parse_dt(end_str);

        if let (Some(s), Some(e)) = (start, end) {
            Ok(serde_json::Value::String(utils::get_time_range_display(s, e)))
        } else {
            println!("!!! [Tera Filter Error] Failed to parse dates: start='{}', end='{}'", start_str, end_str);
            Ok(serde_json::Value::String("".to_string()))
        }
    });
    
    let (layer, io) = SocketIo::new_layer();
    let state = Arc::new(AppState {
        db: pool,
        tera,
        io: io.clone(),
    });

    io.ns("/", |socket: SocketRef| {
        println!("A client connected: {:?}", socket.id);
    });

    let app = Router::new()
        .route("/lantern", get(handlers::client::get_lantern))
        .route("/frontend/index", get(handlers::client::get_frontend_index))
        .route("/frontend/riddles", get(handlers::client::get_riddles))
        .route("/frontend/riddle/:id", get(handlers::client::get_riddle_by_id))
        .route("/q", get(handlers::client::get_q))
        .route("/login", post(handlers::client::login))
        .route("/logout", get(handlers::client::logout).post(handlers::client::logout))
        .route("/guess", post(handlers::client::guess))
        .route("/my/records", get(handlers::client::get_my_records))
        .route("/pro-api/index", get(handlers::admin::get_admin_index))
        .route("/pro-api/users", get(handlers::admin::get_users))
        .route("/pro-api/user/:id", axum::routing::delete(handlers::admin::delete_user))
        .route("/pro-api/riddles", get(handlers::admin::get_riddles).post(handlers::admin::upsert_riddle))
        .route("/pro-api/riddles/import", post(handlers::admin::import_riddles))
        .route("/pro-api/riddle/:id", axum::routing::delete(handlers::admin::delete_riddle))
        .route("/pro-api/leaderboard", get(handlers::admin::get_leaderboard))
        .route("/pro-api/records/export", get(handlers::admin::export_records))
        .route("/pro-api/activity", get(handlers::admin::get_activity).post(handlers::admin::update_activity))
        .fallback(static_handler)
        .layer(axum::extract::DefaultBodyLimit::max(20 * 1024 * 1024)) // 20MB
        .with_state(state)
        .layer(layer)
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], 9000));
    println!(">>> Server listening on {}. Accessible via LAN IP.", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let file_path = if path.is_empty() || path == "index.html" {
        "index.html".to_string()
    } else {
        path.to_string()
    };

    // 【开发模式优化】
    // 优先从本地磁盘读取文件
    #[cfg(debug_assertions)]
    {
        // 尝试从 template 目录读取 (手机端/大屏页面)
        let template_path = std::path::Path::new("template").join(&file_path);
        
        // 尝试从 admin/dist 目录读取 (管理后台，如果用户直接在该目录下运行 Vue build)
        let admin_dist_path = if file_path.starts_with("admin/") {
            // 如果请求是 /admin/xxx，尝试对应到 admin/dist/xxx (假设你在 admin 目录下运行了 build)
            let sub_path = file_path.trim_start_matches("admin/");
            std::path::Path::new("admin").join("dist").join(sub_path)
        } else {
            std::path::Path::new("").to_path_buf()
        };

        let target_path = if template_path.exists() {
            Some(template_path)
        } else if admin_dist_path.exists() {
            Some(admin_dist_path)
        } else {
            None
        };

        if let Some(final_path) = target_path {
            if let Ok(content) = std::fs::read(&final_path) {
                let mime = mime_guess::from_path(&final_path).first_or_octet_stream();
                return Response::builder()
                    .header(header::CONTENT_TYPE, mime.as_ref())
                    .body(Body::from(content))
                    .unwrap();
            }
        }
    }

    // 【生产/打包模式】
    // 从嵌入的资源中读取
    match Asset::get(&file_path) {
        Some(content) => {
            let mime = mime_guess::from_path(&file_path).first_or_octet_stream();
            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data))
                .unwrap()
        }
        None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // 获取 AppData 目录，如果不存在则创建
            let app_data_dir = app.path().app_data_dir().expect("Failed to get AppData directory");
            if !app_data_dir.exists() {
                std::fs::create_dir_all(&app_data_dir).expect("Failed to create AppData directory");
            }
            let db_path = app_data_dir.join("lantern.db");

            // 异步启动 Axum
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    start_backend(db_path).await;
                });
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub mod ax_extract {
    use axum::{
        async_trait,
        extract::{FromRequest, Request},
        Json, Form,
        response::IntoResponse,
    };
    use serde::de::DeserializeOwned;

    pub struct MaybeFormOrJson<T>(pub T);

    #[async_trait]
    impl<T, S> FromRequest<S> for MaybeFormOrJson<T>
    where
        T: DeserializeOwned,
        S: Send + Sync,
    {
        type Rejection = axum::response::Response;

        async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
            let content_type = req
                .headers()
                .get(axum::http::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .unwrap_or("");

            if content_type.starts_with("application/json") {
                let Json(data) = Json::<T>::from_request(req, state)
                    .await
                    .map_err(|e| e.into_response())?;
                Ok(MaybeFormOrJson(data))
            } else {
                let Form(data) = Form::<T>::from_request(req, state)
                    .await
                    .map_err(|e| e.into_response())?;
                Ok(MaybeFormOrJson(data))
            }
        }
    }
}
