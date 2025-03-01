use crate::chain::types::EventContext;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct CreateTransaction {
    pub context: EventContext,
    pub side_effects: Vec<TransactionSideEffect>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct CreateTransactionSideEffect {
    pub side_effects: Vec<TransactionSideEffect>,
    pub transaction_log_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct TransactionSideEffect {
    pub entity_id: Uuid,
    pub entity_type: String,
}
