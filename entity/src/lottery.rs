use async_graphql::Enum;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::{encode::IsNull, error::BoxDynError, postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef}, types::Decimal, Decode, Encode, Postgres, Type};
use uuid::Uuid;
use sqlx::FromRow;

#[derive(Clone, Debug, FromRow, Serialize, Deserialize)]
pub struct LotteryModel {
    pub id: Uuid,
    pub featured: bool,
    pub uid: String,
    pub name: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub ticket_asset: Uuid,
    pub ticket_price: Decimal,
    pub fee_ticket_amount: Decimal,
    pub max_tickets: Option<i32>,
    pub status: LotteryStatus, // Enum to represent the status of the lottery
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Enum, Copy)]
pub enum LotteryStatus {
    Scheduled,
    #[default]
    Ongoing,
    Completed,
    Cancelled,
}

impl ToString for LotteryStatus {
    fn to_string(&self) -> String {
        match self {
            LotteryStatus::Scheduled => "SCHEDULED".to_string(),
            LotteryStatus::Ongoing => "ONGOING".to_string(),
            LotteryStatus::Completed => "COMPLETED".to_string(),
            LotteryStatus::Cancelled => "CANCELLED".to_string(),
        }
    }
}

impl Encode<'_, Postgres> for LotteryStatus {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        let str_value = match self {
            LotteryStatus::Scheduled => "SCHEDULED",
            LotteryStatus::Ongoing => "ONGOING",
            LotteryStatus::Completed => "COMPLETED",
            LotteryStatus::Cancelled => "CANCELLED",
        };
        Encode::<Postgres>::encode(str_value, buf)
    }
}

impl<'r> Decode<'r, Postgres> for LotteryStatus {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let str_value = value.as_str().unwrap_or("");
        match str_value {
            "SCHEDULED" => Ok(LotteryStatus::Scheduled),
            "ONGOING" => Ok(LotteryStatus::Ongoing),
            "COMPLETED" => Ok(LotteryStatus::Completed),
            "CANCELLED" => Ok(LotteryStatus::Cancelled),
            _ => Err(sqlx::Error::Decode(
                format!("Invalid lottery_status value: {}", str_value).into(),
            )
            .into()),
        }
    }
}

impl Type<Postgres> for LotteryStatus {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("VARCHAR")
    }
}