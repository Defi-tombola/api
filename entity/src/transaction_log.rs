use serde::{Deserialize, Serialize};
use sqlx::types::{
    chrono::{DateTime, Utc},
    Uuid,
};
use sqlx::FromRow;

/// Represents a transaction log in the system.
///
/// This model stores details of a transaction log, including the blockchain chain,  
/// transaction hash, block number, and timestamp of the log's creation.
///
/// # Fields
///
/// - `id` - A unique identifier for the transaction log.
/// - `chain` - The blockchain network associated with the transaction (e.g., "Ethereum").
/// - `address` - The address related to the transaction log.
/// - `block_number` - The block number in which the transaction occurred.
/// - `transaction_hash` - The unique identifier for the transaction in the blockchain.
/// - `log_index` - The index of the log entry in the block.
/// - `created_at` - The timestamp when the transaction log was created.
#[derive(Clone, Debug, PartialEq, Eq, FromRow, Serialize, Deserialize)]
pub struct TransactionLogModel {
    pub id: Uuid,
    pub chain: String,
    pub address: String,
    pub block_number: i64,
    pub transaction_hash: String,
    pub log_index: i32,
    pub created_at: DateTime<Utc>,
}
