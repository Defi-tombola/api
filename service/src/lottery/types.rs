use chrono::{DateTime, Utc};
use entity::prelude::LotteryStatus;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateLottery {
    pub name: String,
    pub uid: String, // Onchain UID
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub ticket_price: Decimal,
    pub fee_ticket_amount: Decimal,
    pub ticket_asset: Uuid,
    pub max_tickets: Option<i32>,
    pub status: LotteryStatus,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct UpdateLottery {
    pub name: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub ticket_price: Option<Decimal>,
    pub fee_ticket_amount: Option<Decimal>,
    pub ticket_asset: Option<Uuid>,
    pub max_tickets: Option<i32>,
    pub status: Option<LotteryStatus>,
}