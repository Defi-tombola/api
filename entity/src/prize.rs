use async_graphql::Enum;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Decimal};
use uuid::Uuid;
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};
use sqlx::{Decode, Encode, Postgres, Type};

#[derive(Clone, Debug, PartialEq, FromRow, Serialize, Deserialize)]
pub struct PrizeModel {
    pub id: Uuid,
    pub lottery_id: Uuid,
    pub prize_asset: Uuid,
    pub value: Decimal,
    pub status: PrizeStatus, // Enum to represent the status of the prize
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, Enum, Copy, Eq, PartialEq)]
pub enum PrizeStatus {
    #[default]
    Active,
    Distributed,
    Refunded,
}

impl ToString for PrizeStatus {
    fn to_string(&self) -> String {
        match self {
            PrizeStatus::Active => "ACTIVE".to_string(),
            PrizeStatus::Distributed => "DISTRIBUTED".to_string(),
            PrizeStatus::Refunded => "REFUNDED".to_string(),
        }
    }
}

impl Encode<'_, Postgres> for PrizeStatus {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        let str_value = match self {
            PrizeStatus::Active => "ACTIVE",
            PrizeStatus::Distributed => "DISTRIBUTED",
            PrizeStatus::Refunded => "REFUNDED",
        };
        Encode::<Postgres>::encode(str_value, buf)
    }
}

impl<'r> Decode<'r, Postgres> for PrizeStatus {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let str_value = value.as_str().unwrap_or("");
        match str_value {
            "ACTIVE" => Ok(PrizeStatus::Active),
            "DISTRIBUTED" => Ok(PrizeStatus::Distributed),
            "REFUNDED" => Ok(PrizeStatus::Refunded),
            _ => Err(sqlx::Error::Decode(
                format!("Invalid prize_status value: {}", str_value).into(),
            )
            .into()),
        }
    }
}

impl Type<Postgres> for PrizeStatus {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("VARCHAR")
    }
}