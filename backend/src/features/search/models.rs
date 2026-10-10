use chrono::NaiveDate;
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct SearchHit {
    pub entry_id: Uuid,
    pub date: NaiveDate,
    pub title: String,
    pub snippet: String,
    pub rank: f32,
}
