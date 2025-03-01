use serde::{Deserialize, Serialize};
use sqlx::types::JsonValue;
use sqlx::types::{
    chrono::{DateTime, Utc},
    Uuid,
};
use sqlx::FromRow;

/// Represents the state of a blockchain at a given time.
///
/// # Fields
///
/// - `id` - A unique identifier for the chain state record.
/// - `chain` - The blockchain network associated with this state.
/// - `value` - A JSON object storing the state data. This value will contain block number and addresses to track events.
/// - `updated_at` - The timestamp when the chain state was last updated.
#[derive(Clone, Debug, PartialEq, Eq, FromRow, Serialize, Deserialize)]
pub struct ChainStateModel {
    pub id: Uuid,
    pub chain: String,
    pub value: JsonValue,
    pub updated_at: DateTime<Utc>,
}
