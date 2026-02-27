use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub avatar: Option<String>,
    pub user_code: Option<String>,
    pub token: Option<String>,
    pub register_time: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Riddle {
    pub id: i64,
    pub question: String,
    pub remark: Option<String>,
    pub options_json: String,
    pub answer: String,
    pub add_time: Option<NaiveDateTime>,
    pub is_solved: bool,
    pub solver_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Activity {
    pub id: i64,
    pub name: String,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct GuessRecord {
    pub id: i64,
    pub user_id: i64,
    pub riddle_id: i64,
    pub solve_time: Option<NaiveDateTime>,
    pub is_solved: bool,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct RiddleWithSolver {
    pub id: i64,
    pub question: String,
    pub remark: Option<String>,
    pub options_json: String,
    pub answer: String,
    pub add_time: Option<NaiveDateTime>,
    pub is_solved: bool,
    pub solver_id: Option<i64>,
    pub solver_name: Option<String>,
    pub solver_avatar: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct GuessRecordWithInfo {
    pub id: i64,
    pub user_id: i64,
    pub user_name: Option<String>,
    pub riddle_id: i64,
    pub riddle_question: Option<String>,
    pub riddle_answer: Option<String>,
    pub solve_time: Option<NaiveDateTime>,
    pub is_solved: bool,
}
