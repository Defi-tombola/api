use async_graphql::Enum;
use chrono::{DateTime, Days, Months, NaiveDateTime, Utc};
use serde::Serialize;
use sqlx::types::Decimal;

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize)]
pub enum DateRangeInput {
    OneDay,
    OneWeek,
    OneMonth,
    All,
}

// TODO: Not sure that this belongs here
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize)]
pub enum AssetDatasetAggregation {
    M1,
    M5,
    M15,
    H1,
    H4,
    D1,
}

impl ToString for AssetDatasetAggregation {
    fn to_string(&self) -> String {
        match self {
            AssetDatasetAggregation::M1 => "1min".to_string(),
            AssetDatasetAggregation::M5 => "5min".to_string(),
            AssetDatasetAggregation::M15 => "15min".to_string(),
            AssetDatasetAggregation::H1 => "1hour".to_string(),
            AssetDatasetAggregation::H4 => "4hour".to_string(),
            AssetDatasetAggregation::D1 => "1day".to_string(),
        }
    }
}

impl From<AssetDatasetAggregation> for i64 {
    fn from(value: AssetDatasetAggregation) -> Self {
        match value {
            AssetDatasetAggregation::M1 => 1,
            AssetDatasetAggregation::M5 => 5,
            AssetDatasetAggregation::M15 => 15,
            AssetDatasetAggregation::H1 => 60,
            AssetDatasetAggregation::H4 => 240,
            AssetDatasetAggregation::D1 => 1440,
        }
    }
}

// TODO: Remove that
const SHORT_STEP: &str = r#"(date_trunc('hour', "vault_share"."created_at") +
(((date_part('minute', "vault_share"."created_at")::integer / 10::integer) * 10::integer)
|| ' minutes')::interval)"#;

const LONG_STEP: &str = r#"date_trunc('day', "vault_share"."created_at")"#;

#[derive(Debug)]
pub struct ChartDataset {
    pub key: NaiveDateTime,
    pub value: Decimal,
}

/// Paginated results for a list of items of type T
pub struct PaginatedResults<T> {
    pub items: Vec<T>,
    pub num_pages: u64,
    pub num_items: u64,
}

pub(crate) fn parse_range_to_key_and_query<'a>(
    range: Option<DateRangeInput>,
    created_at: DateTime<Utc>,
) -> (&'a str, Option<DateTime<Utc>>) {
    let (key_trunk, range_query) = match range {
        Some(DateRangeInput::OneDay) => (SHORT_STEP, Utc::now().checked_sub_days(Days::new(1))),
        Some(DateRangeInput::OneWeek) => (SHORT_STEP, Utc::now().checked_sub_days(Days::new(7))),
        Some(DateRangeInput::OneMonth) => {
            (SHORT_STEP, Utc::now().checked_sub_months(Months::new(1)))
        }
        _ => {
            let diff = Utc::now().signed_duration_since(created_at);

            if diff.num_days() > 30 {
                (LONG_STEP, None)
            } else {
                (SHORT_STEP, None)
            }
        }
    };

    (key_trunk, range_query)
}
