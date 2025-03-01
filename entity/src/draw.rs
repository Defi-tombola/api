use async_graphql::Enum;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};
use sqlx::{Decode, Encode, Postgres, Type};

#[derive(Clone, PartialEq, Eq, Debug, FromRow, Serialize, Deserialize)]
pub struct DrawModel {
    pub id: Uuid,
    pub lottery_id: Uuid,
    pub winner: Option<Uuid>,
    pub draw_date: Option<DateTime<Utc>>, // The date the draw was made
    pub status: DrawStatus, // Enum to represent the status of the draw
    pub transaction_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Default, Debug, Eq, PartialEq, Serialize, Deserialize, Copy, Enum)]
pub enum DrawStatus {
    #[default]
    Pending,
    Completed,
    Cancelled,
}

impl ToString for DrawStatus {
    fn to_string(&self) -> String {
        match self {
            DrawStatus::Pending => "PENDING".to_string(),
            DrawStatus::Completed => "COMPLETED".to_string(),
            DrawStatus::Cancelled => "CANCELLED".to_string(),
        }
    }
}

impl Encode<'_, Postgres> for DrawStatus {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        let str_value = match self {
            DrawStatus::Pending => "PENDING",
            DrawStatus::Completed => "COMPLETED",
            DrawStatus::Cancelled => "CANCELLED",
        };
        Encode::<Postgres>::encode(str_value, buf)
    }
}

impl<'r> Decode<'r, Postgres> for DrawStatus {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let str_value = value.as_str().unwrap_or("");
        match str_value {
            "PENDING" => Ok(DrawStatus::Pending),
            "COMPLETED" => Ok(DrawStatus::Completed),
            "CANCELLED" => Ok(DrawStatus::Cancelled),
            _ => Err(sqlx::Error::Decode(
                format!("Invalid draw_status value: {}", str_value).into(),
            )
            .into()),
        }
    }
}

impl Type<Postgres> for DrawStatus {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("VARCHAR")
    }
}