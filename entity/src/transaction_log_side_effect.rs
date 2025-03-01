use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Represents a side effect associated with a transaction log.
///
/// This model tracks side effects related to a transaction, such as changes to entities  
/// within the system triggered by that transaction.
///
/// # Fields
///
/// - `id` - A unique identifier for the transaction log side effect.
/// - `transaction_log_id` - The identifier of the associated transaction log.
/// - `entity_id` - The identifier of the entity affected by the side effect.
/// - `entity_type` - The type of the affected entity (e.g., "account", "asset", etc.).
#[derive(Clone, Debug, PartialEq, Eq, FromRow, Serialize, Deserialize)]
pub struct TransactionLogSideEffectModel {
    pub id: Uuid,
    pub transaction_log_id: Uuid,
    pub entity_id: Uuid,
    pub entity_type: String,
}
