use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateTicket {
    pub lottery_id: Uuid,
    pub account_id: Uuid,
    pub ticket_price: Decimal,
    pub ticket_asset: Uuid,
    pub amount: i32,
    pub transaction_hash: String,
    pub purchased_at: DateTime<Utc>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct UpdateTicket {
    pub lottery_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub amount: Option<i32>,
    pub purchased_at: Option<DateTime<Utc>>,
}