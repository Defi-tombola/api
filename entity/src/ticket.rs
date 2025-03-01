use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Decimal};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, FromRow, Serialize, Deserialize)]
pub struct TicketModel {
    pub id: Uuid,
    pub lottery_id: Uuid,
    pub account_id: Uuid,
    pub ticket_price: Decimal,
    pub ticket_asset: Uuid,
    pub amount: i32, // Amount of the tickets bought
    pub purchased_at: DateTime<Utc>,
    pub transaction_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}