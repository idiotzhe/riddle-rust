use axum::{
    extract::{Path, Query, State, Multipart},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::{Deserialize};
use std::sync::Arc;
use crate::{AppState, models::*, ax_extract::MaybeFormOrJson, utils::get_local_ip};
use chrono::{Local};
use serde_json::json;
use uuid::Uuid;
use std::path::PathBuf;
use tokio::fs;

#[derive(Deserialize)]
pub struct RiddleParams {
    pub page: Option<u32>,
    #[serde(rename = "pageSize")]
    pub page_size: Option<u32>,
    pub exclude_ids: Option<String>,
}

pub async fn get_lantern(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let activity: Option<Activity> = sqlx::query_as("SELECT * FROM activities LIMIT 1")
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

    let mut ctx = tera::Context::new();
    ctx.insert("activity", &activity);
    if let Some(ip) = get_local_ip() {
        ctx.insert("local_ip", &ip);
    }
    
    match state.tera.render("index.html", &ctx) {
        Ok(html) => Html(html).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Template error: {}", e)).into_response(),
    }
}

pub async fn get_frontend_index(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let activity: Option<Activity> = sqlx::query_as("SELECT * FROM activities LIMIT 1")
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

    let mut ctx = tera::Context::new();
    ctx.insert("activity", &activity);
    if let Some(ip) = get_local_ip() {
        ctx.insert("local_ip", &ip);
    }
    
    match state.tera.render("frontend/index.html", &ctx) {
        Ok(html) => Html(html).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Template error: {}", e)).into_response(),
    }
}

pub async fn get_riddles(
    State(state): State<Arc<AppState>>,
    Query(params): Query<RiddleParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(1);
    let offset = (page - 1) * page_size;
    
    let exclude_ids: Vec<i64> = params.exclude_ids
        .unwrap_or_default()
        .split(',')
        .filter_map(|s| s.parse().ok())
        .collect();

    let mut query_str = String::from(
        "SELECT r.*, u.username as solver_name, u.avatar as solver_avatar 
         FROM riddles r 
         LEFT JOIN users u ON r.solver_id = u.id 
         WHERE r.is_solved = 0"
    );

    if !exclude_ids.is_empty() {
        let placeholders: Vec<String> = exclude_ids.iter().map(|_| "?".to_string()).collect();
        query_str.push_str(&format!(" AND r.id NOT IN ({})", placeholders.join(",")));
    }

    query_str.push_str(" LIMIT ? OFFSET ?");

    let mut query = sqlx::query_as::<_, RiddleWithSolver>(&query_str);
    
    for id in &exclude_ids {
        query = query.bind(id);
    }
    query = query.bind(page_size).bind(offset);

    let items = query.fetch_all(&state.db).await.unwrap_or_default();

    if items.is_empty() {
        return Json(json!({ "code": 404, "message": "没有数据" })).into_response();
    }

    let result: Vec<serde_json::Value> = items.into_iter().map(|item| {
        let options: Vec<String> = serde_json::from_str(&item.options_json).unwrap_or_default();
        let mut val = json!(item);
        val["options"] = json!(options);
        val
    }).collect();

    Json(json!({
        "code": 200,
        "message": "success",
        "data": result
    })).into_response()
}

pub async fn get_riddle_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let riddle: Option<RiddleWithSolver> = sqlx::query_as(
        "SELECT r.*, u.username as solver_name, u.avatar as solver_avatar 
         FROM riddles r 
         LEFT JOIN users u ON r.solver_id = u.id 
         WHERE r.id = ?"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    if let Some(r) = riddle {
        let options: Vec<String> = serde_json::from_str(&r.options_json).unwrap_or_default();
        let mut val = json!(r);
        val["options"] = json!(options);
        return Json(json!({ "data": val, "code": 200 })).into_response();
    }
    Json(json!({ "code": 400 })).into_response()
}

pub async fn get_q(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
    jar: CookieJar,
) -> impl IntoResponse {
    let riddle_id = params.get("r_id").and_then(|id| id.parse::<i64>().ok());
    let user_id = jar.get("user_id").and_then(|c| c.value().parse::<i64>().ok());

    if let Some(uid) = user_id {
        let user: Option<User> = sqlx::query_as("SELECT * FROM users WHERE id = ?")
            .bind(uid)
            .fetch_optional(&state.db)
            .await
            .unwrap_or(None);

        if user.is_some() {
            if let Some(rid) = riddle_id {
                let riddle: Option<Riddle> = sqlx::query_as("SELECT * FROM riddles WHERE id = ?")
                    .bind(rid)
                    .fetch_optional(&state.db)
                    .await
                    .unwrap_or(None);
                
                if let Some(r) = riddle {
                    let mut ctx = tera::Context::new();
                    let options: Vec<String> = serde_json::from_str(&r.options_json).unwrap_or_default();
                    let mut riddle_val = json!(r);
                    riddle_val["options"] = json!(options);
                    
                    ctx.insert("riddle", &riddle_val);
                    ctx.insert("user", &user.unwrap());
                    if let Ok(html) = state.tera.render("question.html", &ctx) {
                        return Html(html).into_response();
                    }
                }
            }
        }
    }

    let activity: Option<Activity> = sqlx::query_as("SELECT * FROM activities LIMIT 1")
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);
    let mut ctx = tera::Context::new();
    ctx.insert("activity", &activity);
    ctx.insert("riddle_id", &riddle_id.unwrap_or(0));
    
    match state.tera.render("index.html", &ctx) {
        Ok(html) => Html(html).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Template error: {}", e)).into_response(),
    }
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut username = String::new();
    let mut avatar_path = String::new();

    tracing::debug!("Login attempt started");

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or_default().to_string();
        tracing::debug!("Field name: {}", name);
        if name == "username" {
            username = field.text().await.unwrap_or_default();
            tracing::debug!("Username: {}", username);
        } else if name == "file" {
            let filename = field.file_name().unwrap_or_default().to_string();
            if !filename.is_empty() {
                let data = field.bytes().await.unwrap_or_default();
                let today = Local::now().format("%Y/%m/%d").to_string();
                let mut upload_dir = PathBuf::from("template/avatar");
                upload_dir.push(&today);
                
                if let Err(_) = fs::create_dir_all(&upload_dir).await {
                    continue;
                }

                let ext = std::path::Path::new(&filename)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("png");
                let new_filename = format!("{}.{}", Uuid::new_v4().simple(), ext);
                let file_path = upload_dir.join(&new_filename);
                
                if let Ok(_) = fs::write(file_path, data).await {
                    avatar_path = format!("/avatar/{}/{}", today, new_filename);
                    tracing::debug!("Avatar saved: {}", avatar_path);
                }
            }
        }
    }

    if username.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": "昵称不能为空" }))).into_response();
    }

    // Generate unique code (8-character hex from UUID)
    let user_code = Uuid::new_v4().simple().to_string()[..8].to_uppercase();
    let dummy_token = Uuid::new_v4().to_string();
    let now = get_beijing_now();

    let result = sqlx::query(
        "INSERT INTO users (username, avatar, user_code, token, register_time) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&username)
    .bind(&avatar_path)
    .bind(&user_code)
    .bind(&dummy_token)
    .bind(now)
    .execute(&state.db)
    .await;

    match result {
        Ok(res) => {
            let user_id = res.last_insert_rowid();
            let user: User = sqlx::query_as("SELECT * FROM users WHERE id = ?")
                .bind(user_id)
                .fetch_one(&state.db)
                .await
                .unwrap();

            // Permanent session via cookie (31 days)
            let cookie = Cookie::build(("user_id", user_id.to_string()))
                .path("/")
                .max_age(time::Duration::seconds(31 * 24 * 60 * 60))
                .build();

            (jar.add(cookie), Json(json!({
                "msg": "登录成功",
                "user_info": user
            }))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("数据库错误: {}", e) }))).into_response(),
    }
}

pub async fn logout(jar: CookieJar) -> impl IntoResponse {
    let cookie = Cookie::build(("user_id", ""))
        .path("/")
        .max_age(time::Duration::seconds(0))
        .build();
    (jar.add(cookie), Json(json!({ "msg": "已退出登录" }))).into_response()
}

#[derive(Deserialize)]
pub struct GuessPayload {
    pub riddle_id: serde_json::Value,
    pub answer: String,
}

pub async fn guess(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    payload: MaybeFormOrJson<GuessPayload>,
) -> impl IntoResponse {
    let user_id = match jar.get("user_id").and_then(|c| c.value().parse::<i64>().ok()) {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "未登录", "code": "NOT_LOGGED_IN" }))).into_response(),
    };

    let user: Option<User> = sqlx::query_as("SELECT * FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

    if user.is_none() {
        return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "用户不存在", "code": "USER_NOT_FOUND" }))).into_response();
    }
    let current_user = user.unwrap();

    let GuessPayload { riddle_id: riddle_id_val, answer } = payload.0;
    let user_answer = answer.trim();

    let riddle_id = if let Some(id) = riddle_id_val.as_i64() {
        Some(id)
    } else if let Some(s) = riddle_id_val.as_str() {
        s.parse::<i64>().ok()
    } else {
        None
    };

    if riddle_id.is_none() || user_answer.is_empty() {
        return Json(json!({ "msg": "参数不全", "code": 400 })).into_response();
    }
    let riddle_id = riddle_id.unwrap();

    let activity: Option<Activity> = sqlx::query_as("SELECT * FROM activities LIMIT 1")
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

    if let Some(act) = activity {
        let now = Local::now().naive_local();
        if now < act.start_time {
            return Json(json!({ "msg": "活动尚未开始", "code": 400 })).into_response();
        }
        if now > act.end_time {
            return Json(json!({ "msg": "活动已经结束", "code": 400 })).into_response();
        }
    }

    let riddle: Option<RiddleWithSolver> = sqlx::query_as(
        "SELECT r.*, u.username as solver_name, u.avatar as solver_avatar FROM riddles r LEFT JOIN users u ON r.solver_id = u.id WHERE r.id = ?"
    )
    .bind(riddle_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let riddle = match riddle {
        Some(r) => r,
        None => return Json(json!({ "msg": "题目不存在", "code": 404 })).into_response(),
    };

    if riddle.is_solved {
        return Json(json!({
            "success": false,
            "msg": format!("太可惜了，这道题已经被 {} 抢先猜中了！", riddle.solver_name.unwrap_or_else(|| "别人".to_string())),
            "code": 400
        })).into_response();
    }

    let existing: Option<GuessRecord> = sqlx::query_as("SELECT * FROM guess_records WHERE user_id = ? AND riddle_id = ?")
        .bind(current_user.id)
        .bind(riddle_id)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

    if existing.is_some() {
        return Json(json!({ "success": false, "msg": "你已经猜过该题了！", "code": 400 })).into_response();
    }

    let now_time = get_beijing_now();
    if user_answer.to_lowercase() == riddle.answer.to_lowercase() {
        // Re-check solve status
        let recheck: (bool,) = sqlx::query_as("SELECT is_solved FROM riddles WHERE id = ?")
            .bind(riddle_id)
            .fetch_one(&state.db)
            .await
            .unwrap_or((true,));

        if recheck.0 {
            return Json(json!({ "success": false, "msg": "手慢了，已被抢答！", "code": 400 })).into_response();
        }

        // 执行更新和插入
        let update_res = sqlx::query("UPDATE riddles SET is_solved = 1, solver_id = ? WHERE id = ?")
            .bind(current_user.id)
            .bind(riddle_id)
            .execute(&state.db)
            .await;

        if let Err(e) = update_res {
             return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("更新失败: {}", e) }))).into_response();
        }

        let insert_res = sqlx::query("INSERT INTO guess_records (user_id, riddle_id, is_solved, solve_time) VALUES (?, ?, 1, ?)")
            .bind(current_user.id)
            .bind(riddle_id)
            .bind(now_time)
            .execute(&state.db)
            .await;

        if let Err(_) = insert_res {
            // 如果插入失败（通常是已经存在记录），虽然对用户透明，但我们停止后续动作
            return Json(json!({ "success": true, "msg": "恭喜你！抢答成功！", "code": 200 })).into_response();
        }

        let _ = state.io.emit("riddle_solved", json!({
            "riddle_id": riddle_id,
            "solver_name": current_user.username,
            "solver_avatar": current_user.avatar
        }));

        return Json(json!({ "success": true, "msg": "恭喜你！抢答成功！", "code": 200 })).into_response();
    } else {
        let _ = sqlx::query("INSERT INTO guess_records (user_id, riddle_id, is_solved, solve_time) VALUES (?, ?, 0, ?)")
            .bind(current_user.id)
            .bind(riddle_id)
            .bind(now_time)
            .execute(&state.db)
            .await;
            
        return Json(json!({ "success": false, "msg": "答案不对，请再接再厉！" })).into_response();
    }
}

pub async fn get_my_records(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> impl IntoResponse {
    let user_id = match jar.get("user_id").and_then(|c| c.value().parse::<i64>().ok()) {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "未登录" }))).into_response(),
    };

    let records: Vec<GuessRecordWithInfo> = sqlx::query_as(
        "SELECT gr.*, u.username as user_name, r.question as riddle_question, r.answer as riddle_answer 
         FROM guess_records gr
         JOIN riddles r ON gr.riddle_id = r.id
         JOIN users u ON gr.user_id = u.id
         WHERE gr.user_id = ? 
         ORDER BY gr.solve_time DESC"
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let result: Vec<serde_json::Value> = records.into_iter().map(|rec| {
        let mut val = json!(rec);
        if let Some(st) = rec.solve_time {
            val["solve_time"] = json!(st.format("%Y-%m-%d %H:%M:%S").to_string());
        }
        val
    }).collect();

    Json(result).into_response()
}
