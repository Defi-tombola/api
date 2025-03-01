use crate::chain::ChainClient;
use crate::config::ChainConfig;
use chrono::{DateTime, Utc};
use ethers::prelude::BlockNumber;
use ethers::types::{Address, H256, U256, U64};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize)]
pub struct EventContext {
    pub chain: String,
    pub block_number: U64,
    pub transaction_hash: H256,
    pub log_index: U256,
    pub src_address: Address,
    pub dst_address: Address,
    pub triggered_at: DateTime<Utc>,
}

impl<'de> Deserialize<'de> for EventContext {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize into a `serde_json::Value`
        let value = Value::deserialize(deserializer)?;

        // Extract and convert fields
        let chain = value
            .get("chain")
            .and_then(Value::as_str)
            .ok_or_else(|| serde::de::Error::missing_field("chain"))?
            .to_string();

        let block_number_str = value
            .get("block_number")
            .and_then(Value::as_str)
            .ok_or_else(|| serde::de::Error::missing_field("block_number"))?;

        let block_number = U64::from_str(block_number_str)
            .map_err(|_| serde::de::Error::custom(format!("Invalid U64: {}", block_number_str)))?;

        let transaction_hash_str = value
            .get("transaction_hash")
            .and_then(Value::as_str)
            .ok_or_else(|| serde::de::Error::missing_field("transaction_hash"))?;

        let transaction_hash = H256::from_str(transaction_hash_str).map_err(|_| {
            serde::de::Error::custom(format!("Invalid H256: {}", transaction_hash_str))
        })?;

        let log_index_str = value
            .get("log_index")
            .and_then(Value::as_str)
            .ok_or_else(|| serde::de::Error::missing_field("log_index"))?;

        let log_index = U256::from_str(log_index_str)
            .map_err(|_| serde::de::Error::custom(format!("Invalid U256: {}", log_index_str)))?;

        let src_address_str = value
            .get("src_address")
            .and_then(Value::as_str)
            .ok_or_else(|| serde::de::Error::missing_field("src_address"))?;

        let src_address = Address::from_str(src_address_str).map_err(|_| {
            serde::de::Error::custom(format!("Invalid Address: {}", src_address_str))
        })?;

        let dst_address_str = value
            .get("dst_address")
            .and_then(Value::as_str)
            .ok_or_else(|| serde::de::Error::missing_field("dst_address"))?;

        let dst_address = Address::from_str(dst_address_str).map_err(|_| {
            serde::de::Error::custom(format!("Invalid Address: {}", dst_address_str))
        })?;

        let triggered_at = value
            .get("triggered_at")
            .and_then(Value::as_str)
            .ok_or_else(|| serde::de::Error::missing_field("triggered_at"))?;

        let triggered_at = DateTime::<Utc>::from_str(triggered_at)
            .map_err(|_| serde::de::Error::custom(format!("Invalid DateTime: {}", triggered_at)))?;

        Ok(EventContext {
            chain,
            block_number,
            transaction_hash,
            log_index,
            src_address,
            dst_address,
            triggered_at,
        })
    }
}

pub struct ChainSnapshot {
    pub config: Arc<ChainConfig>,
    pub client: Arc<ChainClient>,
    pub block_number: BlockNumber,
}
