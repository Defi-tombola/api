use error_stack::Result;
use lib::error::Error;
use service::chain::traits::string::ToHexString;
use service::chain_state::types::State;
use service::chain_state::ChainStateService;
use service::config::service::ChainConfig;
use service::prelude::ServiceProvider;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

#[derive(Clone)]
pub struct StateManager {
    inner: Arc<StateManagerInner>,
}

pub struct StateManagerInner {
    chain_state_service: Arc<ChainStateService>,
    config: ChainConfig,
    state_a: RwLock<State>,
    state_b: RwLock<Option<State>>,
}

/// Represents the current state of the indexer.
///
/// Tracks and manages the current block number and addresses being indexed.
impl StateManager {
    pub async fn new(
        config: &ChainConfig,
        service_provider: ServiceProvider,
    ) -> Result<Self, Error> {
        let chain_state_service = service_provider
            .get_service_unchecked::<ChainStateService>()
            .await;

        // Fetch the state from storage or construct it from the config
        let state_a =
            if let Some(stored_state) = chain_state_service.get_state(config.name.clone()).await? {
                let mut address = stored_state
                    .address
                    .clone()
                    .iter()
                    .map(|a| a.to_lowercase())
                    .collect::<HashSet<String>>();

                address.extend(
                    config
                        .contracts
                        .iter()
                        .map(|c| c.1.to_hex_string().to_lowercase()),
                );

                State {
                    address,
                    ..stored_state.clone()
                }
            } else {
                State {
                    block_number: config.block_number as u64,
                    address: config
                        .contracts
                        .iter()
                        .map(|c| c.1.to_hex_string().to_lowercase())
                        .collect(),
                }
            };

        Ok(Self {
            inner: Arc::new(StateManagerInner {
                config: config.clone(),
                chain_state_service,
                state_a: RwLock::new(state_a),
                state_b: RwLock::new(None),
            }),
        })
    }

    /// Fetches the indexer's current state.
    ///
    /// Returns [`State`] with current processing block number and addresses.
    pub async fn current(&self) -> State {
        self.inner.state_a.read().await.clone()
    }

    /// Fetches the indexer's next state.
    ///
    /// Returns next [`State`] with block number and addresses to process next.
    pub async fn next(&self) -> Option<State> {
        let mut state_a = self.inner.state_a.write().await;
        let mut state_b = self.inner.state_b.write().await;

        if let Some(state_b_inner) = &*state_b {
            let new_state = State {
                block_number: state_a.block_number,
                ..state_b_inner.clone()
            };

            *state_a = new_state.clone();
            *state_b = None;

            return Some(new_state);
        }

        None
    }

    /// Persist the state to the database
    pub async fn save(&self) -> Result<(), Error> {
        let state_a = self.inner.state_a.read().await;
        let state_b = self.inner.state_b.read().await;
        let state = state_b.as_ref().unwrap_or(&*state_a);

        info!("Saving state for indexer chain {}", self.inner.config.name);

        self.inner
            .chain_state_service
            .save_state(self.inner.config.name.clone(), state)
            .await?;

        Ok(())
    }

    /// Set the block number of processed block by the indexer
    pub async fn set_block_number(&self, block_number: u64) -> Result<(), Error> {
        let mut state_a = self.inner.state_a.write().await;
        state_a.block_number = block_number;

        let mut state_b = self.inner.state_b.write().await;
        if let Some(ref mut state_b_inner) = *state_b {
            state_b_inner.block_number = block_number;
        }

        Ok(())
    }

    /// Add an address to the indexer
    pub async fn add_address(&self, address: String) -> Result<(), Error> {
        let state_a = self.inner.state_a.read().await;
        let mut state_b = self.inner.state_b.write().await;

        let state = state_b.as_ref().unwrap_or(&*state_a);

        *state_b = Some(State {
            block_number: state_a.block_number,
            address: std::iter::once(address)
                .chain(state.address.clone())
                .collect(),
        });

        Ok(())
    }

    /// Remove an address from the indexer
    #[allow(dead_code)]
    pub async fn remove_address(&self, address: &str) -> Result<(), Error> {
        let state_a = self.inner.state_a.read().await;
        let mut state_b = self.inner.state_b.write().await;

        let state = state_b.as_ref().unwrap_or(&*state_a);

        *state_b = Some(State {
            block_number: state_a.block_number,
            address: state
                .address
                .iter()
                .filter(|a| a != &address)
                .cloned()
                .collect(),
        });

        Ok(())
    }
}
