use chrono::{DateTime, Utc};
use entity::draw::DrawStatus;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateDraw {
    pub lottery_id: Uuid,
    pub status: DrawStatus,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct UpdateDraw {
    pub draw_date: Option<DateTime<Utc>>,
    pub winner: Option<Uuid>,
    pub transaction_hash: Option<String>,
    pub status: Option<DrawStatus>,
}