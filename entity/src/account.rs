use serde::{Deserialize, Serialize};
use sqlx::types::{chrono::{DateTime, Utc}, Uuid};
use sqlx::FromRow;

#[derive(Clone, Debug, PartialEq, FromRow, Serialize, Deserialize)]
pub struct AccountModel {
    pub id: Uuid,
    pub address: String,
    pub avatar: Option<String>,
    pub name: Option<String>,
    pub twitter: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}