use chrono::{DateTime, Utc};
use entity::{prelude::LotteryStatus, prize::PrizeStatus};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreatePrize {
    pub lottery_id: Uuid,
    pub prize_asset: Uuid,
    pub value: Decimal,
    pub status: PrizeStatus,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct UpdatePrize {
    pub lottery_id: Option<Uuid>,
    pub prize_asset: Option<Uuid>,
    pub value: Option<Decimal>,
    pub status: Option<PrizeStatus>,
}