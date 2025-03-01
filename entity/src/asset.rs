use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};
use sqlx::types::{Decimal, Uuid};
use sqlx::{Decode, Encode, FromRow, Postgres, Type};

#[derive(Clone, Debug, PartialEq, Eq, FromRow, Serialize, Deserialize)]
pub struct AssetModel {
    pub id: Uuid,
    pub name: String,
    pub address: String,
    pub asset_type: AssetType,
    pub symbol: String,
    pub shadow_symbol: Option<String>,
    pub decimals: Option<i16>,
    pub deprecated: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssetType {
    #[default]
    Erc20,
    Unknown,
}

impl ToString for AssetType {
    fn to_string(&self) -> String {
        match self {
            AssetType::Erc20 => "ERC20".to_string(),
            AssetType::Unknown => "UNKNOWN".to_string(),
        }
    }
}

impl<'de> Deserialize<'de> for AssetType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?.to_uppercase();
        match s.as_str() {
            "ERC20" => Ok(AssetType::Erc20),
            "UNKNOWN" => Ok(AssetType::Unknown),
            _ => Err(serde::de::Error::unknown_variant(
                &s,
                &["ERC20", "UNKNOWN"],
            )),
        }
    }
}

impl Encode<'_, Postgres> for AssetType {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        let str_value = match self {
            AssetType::Erc20 => "ERC20",
            AssetType::Unknown => "UNKNOWN",
        };
        Encode::<Postgres>::encode(str_value, buf)
    }
}

impl<'r> Decode<'r, Postgres> for AssetType {
    fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let str_value = value.as_str().unwrap_or("");
        match str_value {
            "ERC20" => Ok(AssetType::Erc20),
            "UNKNOWN" => Ok(AssetType::Unknown),
            _ => Err(sqlx::Error::Decode(
                format!("Invalid asset_type value: {}", str_value).into(),
            )
            .into()),
        }
    }
}

impl Type<Postgres> for AssetType {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("VARCHAR")
    }
}

impl From<AssetType> for String {
    fn from(value: AssetType) -> Self {
        match value {
            AssetType::Erc20 => "ERC20".to_string(),
            AssetType::Unknown => "UNKNOWN".to_string(),
        }
    }
}