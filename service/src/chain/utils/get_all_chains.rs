use crate::chain::init_client;
use crate::chain::types::ChainSnapshot;
use crate::chain::utils::get_block::get_block_by_timestamp;
use crate::config::ConfigService;
use ethers::prelude::{BlockNumber, U256};
use futures_util::future::join_all;
use lib::error::Error;
use std::collections::HashMap;
use std::sync::Arc;

/// Gets all the available chains from the config service.
pub async fn get_all_chains_snapshot(
    config_service: Arc<ConfigService>,
    timestamp: U256,
) -> error_stack::Result<Arc<HashMap<String, ChainSnapshot>>, Error> {
    let chains_config = config_service.chains.clone();
    let chains = join_all(chains_config.iter().map(|c| async {
        let client = init_client(c).expect("Failed to init RPC provider");

        let block_number = get_block_by_timestamp(client.clone(), c, timestamp)
            .await
            .expect("Failed to get block number");

        ChainSnapshot {
            config: Arc::from(c.clone()),
            client,
            block_number: block_number
                .map(BlockNumber::Number)
                .unwrap_or(BlockNumber::Latest),
        }
    }))
    .await;

    let chains: Arc<HashMap<String, ChainSnapshot>> = Arc::new(
        chains
            .into_iter()
            .map(|c| (c.config.name.clone(), c))
            .collect(),
    );

    Ok(chains)
}
