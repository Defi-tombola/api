use crate::chain_state::store::ChainStateStore;
use crate::chain_state::types::{CreateChainState, State, UpdateChainState};
use crate::prelude::{ServiceProvider, StoreService};
use crate::services::ServiceFactory;
use async_trait::async_trait;
use error_stack::Result;
use lib::error::Error;
use std::sync::Arc;

pub mod store;
pub mod types;

pub struct ChainStateService {
    store: Arc<StoreService>,
}

impl ChainStateService {
    pub fn new(store: Arc<StoreService>) -> Self {
        Self { store }
    }

    pub async fn get_state(&self, chain_name: String) -> Result<Option<State>, Error> {
        let chain_state =
            store::ChainStateStore::try_find_by_chain_name(self.store.read(), chain_name).await?;

        let state =
            match chain_state.map(|state| serde_json::from_value::<types::State>(state.value)) {
                Some(Ok(state)) => Some(state),
                Some(Err(_e)) => None,
                None => None,
            };

        Ok(state)
    }

    pub async fn save_state(&self, chain_name: String, state: &State) -> Result<(), Error> {
        let mut db_tx = self.store.begin_transaction().await?;

        // TODO: Should we search for input chain everytime we do a save state or add this logic to the store?
        let chain_model =
            ChainStateStore::try_find_by_chain_name(db_tx.as_mut(), chain_name.clone()).await?;

        if let Some(chain_state) = chain_model {
            ChainStateStore::update_by_chain_name(
                db_tx.as_mut(),
                chain_state.chain,
                UpdateChainState {
                    value: serde_json::to_value(state).unwrap(),
                    updated_at: chrono::Utc::now(),
                },
            )
            .await?;
        } else {
            ChainStateStore::create(
                db_tx.as_mut(),
                CreateChainState {
                    chain: chain_name.clone(),
                    value: serde_json::to_value(state).unwrap(),
                    updated_at: chrono::Utc::now(),
                },
            )
            .await?;
        }
        self.store.commit_transaction(db_tx).await?;

        Ok(())
    }
}

#[async_trait]
impl ServiceFactory for ChainStateService {
    async fn factory(services: ServiceProvider) -> error_stack::Result<Self, Error>
    where
        Self: Sized,
    {
        let store = services.get_service_unchecked::<StoreService>().await;

        Ok(Self::new(store))
    }
}
