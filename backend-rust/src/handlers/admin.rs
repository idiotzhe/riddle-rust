use axum::{
    extract::{Path, Query, State, Multipart},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
};
use serde::{Deserialize};
use std::sync::Arc;
use crate::{AppState, models::*, utils::get_beijing_now};
use chrono::{Local, NaiveDateTime};
use serde_json::json;
use calamine::{Reader, Xlsx};
use std::io::Cursor;

#[derive(Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    #[serde(rename = "pageSize")]
    pub page_size: Option<u32>,
    pub keyword: Option<String>,
}

pub async fn get_admin_index(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let ctx = tera::Context::new();
    match state.tera.render("admin/index.html", &ctx) {
        Ok(html) => Html(html).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Template error: {}", e)).into_response(),
    }
}

pub async fn get_users(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(10);
    let offset = (page - 1) * page_size;
    let keyword = params.keyword.unwrap_or_default();

    let mut count_query = String::from("SELECT COUNT(*) as total FROM users");
    let mut data_query = String::from("SELECT * FROM users");
    
    if !keyword.is_empty() {
        let clause = format!(" WHERE username LIKE '%{}%'", keyword);
        count_query.push_str(&clause);
        data_query.push_str(&clause);
    }

    data_query.push_str(&format!(" LIMIT {} OFFSET {}", page_size, offset));

    let total: (i64,) = sqlx::query_as(&count_query).fetch_one(&state.db).await.unwrap_or((0,));
    let items: Vec<User> = sqlx::query_as(&data_query).fetch_all(&state.db).await.unwrap_or_default();

    let list: Vec<serde_json::Value> = items.into_iter().map(|u| {
        let mut val = json!(u);
        if let Some(t) = u.register_time {
            val["register_time"] = json!(t.format("%Y-%m-%d %H:%M:%S").to_string());
        }
        val
    }).collect();

    Json(json!({
        "code": 200,
        "message": "success",
        "data": {
            "total": total.0,
            "page": page,
            "totalPages": (total.0 as f64 / page_size as f64).ceil() as i64,
            "list": list
        }
    }))
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    // 先删除该用户的答题记录
    let _ = sqlx::query("DELETE FROM guess_records WHERE user_id = ?").bind(id).execute(&state.db).await;
    
    // 如果该用户是某些灯谜的获胜者，清除灯谜表中的获胜者信息
    let _ = sqlx::query("UPDATE riddles SET is_solved = 0, solver_id = NULL WHERE solver_id = ?").bind(id).execute(&state.db).await;

    let result = sqlx::query("DELETE FROM users WHERE id = ?").bind(id).execute(&state.db).await;
    match result {
        Ok(res) if res.rows_affected() > 0 => Json(json!({ "code": 200, "message": "删除成功" })).into_response(),
        _ => Json(json!({ "code": 404, "message": "用户不存在或已被删除" })).into_response(),
    }
}

pub async fn get_riddles(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(10);
    let offset = (page - 1) * page_size;
    let keyword = params.keyword.unwrap_or_default();

    let mut count_query = String::from("SELECT COUNT(*) as total FROM riddles");
    let mut data_query = String::from("SELECT r.*, u.username as solver_name, u.avatar as solver_avatar FROM riddles r LEFT JOIN users u ON r.solver_id = u.id");
    
    if !keyword.is_empty() {
        let clause = format!(" WHERE r.question LIKE '%{}%'", keyword);
        count_query.push_str(&clause);
        data_query.push_str(&clause);
    }

    data_query.push_str(&format!(" ORDER BY r.add_time DESC LIMIT {} OFFSET {}", page_size, offset));

    let total: (i64,) = sqlx::query_as(&count_query).fetch_one(&state.db).await.unwrap_or((0,));
    let items: Vec<RiddleWithSolver> = sqlx::query_as(&data_query).fetch_all(&state.db).await.unwrap_or_default();

    let list: Vec<serde_json::Value> = items.into_iter().map(|r| {
        let options: Vec<String> = serde_json::from_str(&r.options_json).unwrap_or_default();
        let mut val = json!(r);
        val["options"] = json!(options);
        if let Some(t) = r.add_time {
            val["add_time"] = json!(t.format("%Y-%m-%d %H:%M:%S").to_string());
        }
        val
    }).collect();

    Json(json!({
        "code": 200,
        "message": "success",
        "data": {
            "total": total.0,
            "page": page,
            "totalPages": (total.0 as f64 / page_size as f64).ceil() as i64,
            "list": list
        }
    }))
}

#[derive(Deserialize)]
pub struct RiddleUpsertPayload {
    pub id: Option<i64>,
    pub question: Option<String>,
    pub answer: Option<String>,
    pub remark: Option<String>,
    pub options: Option<Vec<String>>,
    pub reset_status: Option<serde_json::Value>,
}

pub async fn upsert_riddle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RiddleUpsertPayload>,
) -> impl IntoResponse {
    if let Some(id) = payload.id {
        let riddle: Option<Riddle> = sqlx::query_as("SELECT * FROM riddles WHERE id = ?").bind(id).fetch_optional(&state.db).await.unwrap_or(None);
        if let Some(r) = riddle {
            let question = payload.question.unwrap_or(r.question);
            let answer = payload.answer.unwrap_or(r.answer);
            let remark = payload.remark.unwrap_or(r.remark.unwrap_or_default());
            let options_json = payload.options.map(|o| serde_json::to_string(&o).unwrap()).unwrap_or(r.options_json);
            
            let mut is_solved = r.is_solved;
            let mut solver_id = r.solver_id;
            
            if let Some(reset) = payload.reset_status {
                if reset == true || reset == "true" {
                    is_solved = false;
                    solver_id = None;
                }
            }

            sqlx::query("UPDATE riddles SET question = ?, answer = ?, remark = ?, options_json = ?, is_solved = ?, solver_id = ? WHERE id = ?")
                .bind(question).bind(answer).bind(remark).bind(options_json).bind(is_solved).bind(solver_id).bind(id)
                .execute(&state.db).await.unwrap_or_default();
            
            // 获取更新后的数据，使用 fetch_optional 避免 Panic
            let updated: Option<RiddleWithSolver> = sqlx::query_as("SELECT r.*, u.username as solver_name, u.avatar as solver_avatar FROM riddles r LEFT JOIN users u ON r.solver_id = u.id WHERE r.id = ?")
                .bind(id).fetch_optional(&state.db).await.unwrap_or(None);
            
            if let Some(upd) = updated {
                let options: Vec<String> = serde_json::from_str(&upd.options_json).unwrap_or_default();
                let mut val = json!(upd);
                val["options"] = json!(options);
                if let Some(t) = upd.add_time {
                    val["add_time"] = json!(t.format("%Y-%m-%d %H:%M:%S").to_string());
                }
                return Json(json!({ "code": 200, "message": "更新成功", "data": val })).into_response();
            }
            return Json(json!({ "code": 200, "message": "更新成功(未获取到回显数据)" })).into_response();
        }
        return Json(json!({ "code": 404, "message": "灯谜不存在" })).into_response();
    } else {
        let options_json = serde_json::to_string(&payload.options.unwrap_or_default()).unwrap();
        let now = get_beijing_now();
        let result = sqlx::query("INSERT INTO riddles (question, answer, remark, options_json, add_time) VALUES (?, ?, ?, ?, ?)")
            .bind(payload.question.unwrap_or_default())
            .bind(payload.answer.unwrap_or_default())
            .bind(payload.remark.unwrap_or_default())
            .bind(options_json)
            .bind(now)
            .execute(&state.db).await.unwrap();
        
        let inserted: Riddle = sqlx::query_as("SELECT * FROM riddles WHERE id = ?").bind(result.last_insert_rowid()).fetch_one(&state.db).await.unwrap();
        let mut val = json!(inserted);
        if let Some(t) = inserted.add_time {
            val["add_time"] = json!(t.format("%Y-%m-%d %H:%M:%S").to_string());
        }
        return Json(json!({ "code": 200, "message": "创建成功", "data": val })).into_response();
    }
}

pub async fn delete_riddle(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    sqlx::query("DELETE FROM guess_records WHERE riddle_id = ?").bind(id).execute(&state.db).await.unwrap_or_default();
    let result = sqlx::query("DELETE FROM riddles WHERE id = ?").bind(id).execute(&state.db).await;
    match result {
        Ok(res) if res.rows_affected() > 0 => Json(json!({ "code": 200, "message": "删除成功" })).into_response(),
        _ => Json(json!({ "code": 404, "message": "不存在" })).into_response(),
    }
}

pub async fn get_leaderboard(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(10);
    let offset = (page - 1) * page_size;
    let keyword = params.keyword.unwrap_or_default();

    let mut count_query = String::from("SELECT COUNT(*) as total FROM guess_records gr JOIN users u ON gr.user_id = u.id WHERE gr.is_solved = 1");
    let mut data_query = String::from("SELECT gr.*, u.username as user_name, r.question as riddle_question, r.answer as riddle_answer FROM guess_records gr JOIN users u ON gr.user_id = u.id JOIN riddles r ON gr.riddle_id = r.id WHERE gr.is_solved = 1");
    
    if !keyword.is_empty() {
        let clause = format!(" AND u.username LIKE '%{}%'", keyword);
        count_query.push_str(&clause);
        data_query.push_str(&clause);
    }

    data_query.push_str(&format!(" ORDER BY gr.solve_time DESC LIMIT {} OFFSET {}", page_size, offset));

    let total: (i64,) = sqlx::query_as(&count_query).fetch_one(&state.db).await.unwrap_or((0,));
    let items: Vec<GuessRecordWithInfo> = sqlx::query_as(&data_query).fetch_all(&state.db).await.unwrap_or_default();

    let list: Vec<serde_json::Value> = items.into_iter().map(|rec| {
        let mut val = json!(rec);
        if let Some(t) = rec.solve_time {
            val["solve_time"] = json!(t.format("%Y-%m-%d %H:%M:%S").to_string());
        }
        val
    }).collect();

    Json(json!({
        "code": 200,
        "message": "success",
        "data": {
            "total": total.0,
            "page": page,
            "totalPages": (total.0 as f64 / page_size as f64).ceil() as i64,
            "list": list
        }
    }))
}

#[derive(Deserialize)]
pub struct ExportParams {
    pub keyword: Option<String>,
    pub save_locally: Option<bool>,
}

pub async fn export_records(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(params): Query<ExportParams>,
) -> impl IntoResponse {
    let keyword = params.keyword.unwrap_or_default();
    
    // 自动检测环境：如果是从 tauri.localhost 发来的请求，默认开启本地保存
    let origin = headers.get("origin").and_then(|v| v.to_str().ok()).unwrap_or("");
    let is_tauri = origin.contains("tauri.localhost") || params.save_locally.unwrap_or(false);

    // 【核心修改】导出接口强制忽略分页，查询全部中奖记录
    let mut data_query = String::from("SELECT gr.*, u.username as user_name, r.question as riddle_question, r.answer as riddle_answer FROM guess_records gr JOIN users u ON gr.user_id = u.id JOIN riddles r ON gr.riddle_id = r.id WHERE gr.is_solved = 1");
    
    if !keyword.is_empty() {
        let clause = format!(" AND u.username LIKE '%{}%'", keyword);
        data_query.push_str(&clause);
    }

    // 依然保留按时间倒序，但不加 LIMIT 和 OFFSET
    data_query.push_str(" ORDER BY gr.solve_time DESC");

    let items: Vec<GuessRecordWithInfo> = sqlx::query_as(&data_query).fetch_all(&state.db).await.unwrap_or_default();

    // Generate CSV content with BOM
    let mut csv = String::from("\u{feff}记录ID,中奖用户,答对灯谜,谜底,中奖时间\n");
    for item in items {
        let time_str = item.solve_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_default();
        csv.push_str(&format!(
            "{},{},\"{}\",\"{}\",{}\n",
            item.id,
            item.user_name.unwrap_or_default(),
            item.riddle_question.unwrap_or_default().replace("\"", "\"\""),
            item.riddle_answer.unwrap_or_default().replace("\"", "\"\""),
            time_str
        ));
    }

    // --- 桌面端自动保存逻辑 ---
    if is_tauri {
        let filename = format!("灯谜中奖记录_{}.csv", Local::now().format("%Y%m%d_%H%M%S"));
        
        // 1. 尝试获取程序同级目录
        let exe_dir = std::env::current_exe().ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        let save_path = exe_dir.join(&filename);

        // 执行写入
        match std::fs::write(&save_path, &csv) {
            Ok(_) => {
                return Json(json!({
                    "code": 200,
                    "message": format!("导出成功！文件已保存至程序所在目录：\n{}", filename),
                    "data": save_path.to_string_lossy()
                })).into_response();
            },
            Err(e) => {
                // 如果权限不足，尝试写到用户的桌面（简单兜底）
                return Json(json!({
                    "code": 500,
                    "message": format!("导出失败：文件写入错误 ({})。请尝试以管理员身份运行程序。", e)
                })).into_response();
            }
        }
    }

    // --- 标准 Web 下载逻辑 ---
    use axum::http::header;
    (
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"records.csv\""),
        ],
        csv,
    ).into_response()
}

pub async fn get_activity(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let act: Option<Activity> = sqlx::query_as("SELECT * FROM activities LIMIT 1").fetch_optional(&state.db).await.unwrap_or(None);
    if let Some(a) = act {
        return Json(json!(a)).into_response();
    }
    
    let now = Local::now().naive_local();
    let tomorrow = now + chrono::Duration::days(1);
    sqlx::query("INSERT INTO activities (name, start_time, end_time) VALUES (?, ?, ?)")
        .bind("元宵猜灯谜").bind(now).bind(tomorrow).execute(&state.db).await.unwrap();
    
    let act: Activity = sqlx::query_as("SELECT * FROM activities LIMIT 1").fetch_one(&state.db).await.unwrap();
    Json(json!(act)).into_response()
}

#[derive(Deserialize)]
pub struct ActivityPayload {
    pub name: Option<String>,
    pub start_time: String,
    pub end_time: String,
}

pub async fn update_activity(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ActivityPayload>,
) -> impl IntoResponse {
    let start = NaiveDateTime::parse_from_str(&payload.start_time, "%Y-%m-%d %H:%M:%S").unwrap_or_else(|_| Local::now().naive_local());
    let end = NaiveDateTime::parse_from_str(&payload.end_time, "%Y-%m-%d %H:%M:%S").unwrap_or_else(|_| Local::now().naive_local());
    let name = payload.name.unwrap_or_else(|| "元宵猜灯谜".to_string());

    let act: Option<Activity> = sqlx::query_as("SELECT * FROM activities LIMIT 1").fetch_optional(&state.db).await.unwrap_or(None);
    if let Some(a) = act {
        sqlx::query("UPDATE activities SET name = ?, start_time = ?, end_time = ? WHERE id = ?")
            .bind(&name).bind(start).bind(end).bind(a.id).execute(&state.db).await.unwrap();
    } else {
        sqlx::query("INSERT INTO activities (name, start_time, end_time) VALUES (?, ?, ?)")
            .bind(&name).bind(start).bind(end).execute(&state.db).await.unwrap();
    }
    
    let updated: Activity = sqlx::query_as("SELECT * FROM activities LIMIT 1").fetch_one(&state.db).await.unwrap();
    Json(json!(updated)).into_response()
}

pub async fn import_riddles(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("file") {
            let data = field.bytes().await.unwrap_or_default();
            let mut excel: Xlsx<_> = match Xlsx::new(Cursor::new(data)) {
                Ok(e) => e,
                Err(e) => return Json(json!({ "code": 400, "msg": format!("Excel error: {}", e) })).into_response(),
            };
            if let Some(Ok(range)) = excel.worksheet_range_at(0) {
                let mut count = 0;
                let now = get_beijing_now();
                
                let mut rows = range.rows();
                let headers: Vec<String> = match rows.next() {
                    Some(r) => r.iter().map(|c| c.to_string()).collect(),
                    None => return Json(json!({ "code": 400, "msg": "Empty excel" })).into_response(),
                };
                
                for row in rows {
                    let mut question = String::new();
                    let mut answer = String::new();
                    let mut remark = String::new();
                    let mut options = Vec::new();

                    for (i, cell) in row.iter().enumerate() {
                        let header = headers.get(i).map(|s| s.as_str()).unwrap_or("");
                        match header {
                            "灯谜题目" => question = cell.to_string(),
                            "正确答案" => answer = cell.to_string(),
                            "描述" => remark = cell.to_string(),
                            h if h.contains("选项") => {
                                let val = cell.to_string();
                                if !val.is_empty() {
                                    options.push(val);
                                }
                            }
                            _ => {}
                        }
                    }

                    if !question.is_empty() && !answer.is_empty() {
                        {
                            use rand::seq::SliceRandom;
                            let mut rng = rand::thread_rng();
                            options.shuffle(&mut rng);
                        }

                        let options_json = serde_json::to_string(&options).unwrap();
                        sqlx::query("INSERT INTO riddles (question, answer, remark, options_json, add_time) VALUES (?, ?, ?, ?, ?)")
                            .bind(question).bind(answer).bind(remark).bind(options_json).bind(now)
                            .execute(&state.db).await.unwrap();
                        count += 1;
                    }
                }
                return Json(json!({ "code": 200, "msg": format!("灯谜导入成功: 共 {} 条", count) })).into_response();
            }
        }
    }
    Json(json!({ "code": 400, "msg": "未上传文件或数据格式错误" })).into_response()
}
