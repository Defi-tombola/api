use chrono::{DateTime, Utc};
use sqlx::types::JsonValue;

#[derive(Debug)]
pub struct CreateChainState {
    pub chain: String,
    pub value: JsonValue,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct UpdateChainState {
    pub value: JsonValue,
    pub updated_at: DateTime<Utc>,
}

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub block_number: u64,
    pub address: HashSet<String>,
}

impl State {
    pub fn address(&self) -> Vec<String> {
        self.address.iter().cloned().collect()
    }
}
