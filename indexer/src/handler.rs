use async_trait::async_trait;
use chrono::{DateTime, Utc};
use error_stack::Result;
use ethers::types::{Address, H256, U256, U64};
use lib::error::Error;
use service::{
    chain::{provider::ChainProvider, types::EventContext},
    services::ServiceProvider,
    store::service::DatabaseTransaction,
};

use crate::{state::StateManager, stream::ChainEvent};

/// Encapsulates the data payload for handling events of type `T`.
#[derive(Clone)]
pub(crate) struct HandlerPayload<Kind> {
    pub block_number: U64,
    pub log_index: U256,
    pub transaction_hash: H256,
    pub src_address: Address,
    pub dst_address: Address,
    pub kind: Kind,
    pub triggered_at: DateTime<Utc>, // Date Time (UTC) when the block of the event was mined.
}

impl<Kind> From<(ChainEvent, Kind)> for HandlerPayload<Kind> {
    fn from((event, kind): (ChainEvent, Kind)) -> Self {
        Self {
            block_number: event.block_number,
            log_index: event.log_index,
            transaction_hash: event.transaction_hash,
            src_address: event.src_address,
            dst_address: event.dst_address,
            kind,
            triggered_at: event.triggered_at,
        }
    }
}

impl<Kind> HandlerPayload<Kind> {
    pub fn get_context<Provider: ChainProvider>(&self, provider: &Provider) -> EventContext {
        EventContext {
            chain: provider.name(),
            block_number: self.block_number,
            transaction_hash: self.transaction_hash,
            log_index: self.log_index,
            src_address: self.src_address,
            dst_address: self.dst_address,
            triggered_at: self.triggered_at,
        }
    }
}

/// Defines a handler for processing events of a specific kind.
///
/// # Example
///
/// ```rust
/// #[async_trait]
/// impl<Provider> Handler<AssetAdded> for Provider
/// where
///     Provider: ChainProvider + Send + Sync,
/// {
///     async fn handle(&self, payload: HandlerPayload<AssetAdded>) -> Result<(), Error> {
///         // Process the event...
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub(crate) trait Handler<Kind>
where
    Self: ChainProvider,
{
    async fn handle(
        &self,
        payload: HandlerPayload<Kind>,
        services: ServiceProvider,
        state: StateManager,
        db_tx: &mut DatabaseTransaction<'_>,
    ) -> Result<(), Error>;
}
