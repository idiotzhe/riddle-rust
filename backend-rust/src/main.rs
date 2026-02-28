use axum::{
    routing::{get, post},
    Router,
    response::{Html, IntoResponse, Response},
    body::Body,
    http::{header, StatusCode, Uri},
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use sqlx::sqlite::SqlitePoolOptions;
use tera::Tera;
use std::sync::Arc;
use socketioxide::{SocketIo, extract::SocketRef};
use rust_embed::RustEmbed;

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

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "backend_rust=debug,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 数据库路径：使用 mode=rwc，如果文件不存在则自动创建
    let db_url = "sqlite:lantern.db?mode=rwc";
    
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await
        .expect("Failed to connect to lantern.db");

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

    // 初始化 Tera 并加载嵌入的模板
    let mut tera = Tera::default();
    let mut templates = Vec::new();
    for file in Asset::iter() {
        if file.ends_with(".html") {
            if let Some(content) = Asset::get(&file) {
                let template_str = std::str::from_utf8(content.data.as_ref()).expect("UTF-8 error");
                templates.push((file.to_string(), template_str.to_string()));
            }
        }
    }
    tera.add_raw_templates(templates).expect("Failed to load embedded templates");

    tera.register_filter("get_time_range_display", move |value: &serde_json::Value, _args: &std::collections::HashMap<String, serde_json::Value>| {
        let start_str = value.get("start_time").and_then(|v| v.as_str()).unwrap_or("");
        let end_str = value.get("end_time").and_then(|v| v.as_str()).unwrap_or("");
        
        let start = chrono::NaiveDateTime::parse_from_str(start_str, "%Y-%m-%dT%H:%M:%S").ok()
            .or_else(|| chrono::NaiveDateTime::parse_from_str(start_str, "%Y-%m-%d %H:%M:%S").ok());
        let end = chrono::NaiveDateTime::parse_from_str(end_str, "%Y-%m-%dT%H:%M:%S").ok()
            .or_else(|| chrono::NaiveDateTime::parse_from_str(end_str, "%Y-%m-%d %H:%M:%S").ok());

        if let (Some(s), Some(e)) = (start, end) {
            Ok(serde_json::Value::String(utils::get_time_range_display(s, e)))
        } else {
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
        .with_state(state)
        .layer(layer)
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], 9000));
    println!("Standalone Server running at http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// 静态文件处理器：从嵌入的资源中读取
async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    if path.is_empty() || path == "index.html" {
        return Html(Asset::get("index.html").map(|content| std::str::from_utf8(content.data.as_ref()).unwrap().to_string()).unwrap_or_default()).into_response();
    }

    match Asset::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data))
                .unwrap()
        }
        None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
    }
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
