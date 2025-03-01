use crate::{
    events::FeeCollected,
    handler::{Handler, HandlerPayload},
    state::StateManager,
};
use async_trait::async_trait;
use error_stack::{Report, Result};
use lib::error::Error;
use service::{chain::provider::ChainProvider, store::service::{DatabaseTransaction, StoreService}};
use service::services::ServiceProvider;
use tracing::{info, warn};

#[async_trait]
impl<Provider> Handler<FeeCollected> for Provider
where
    Provider: ChainProvider,
{
    async fn handle(
        &self,
        payload: HandlerPayload<FeeCollected>,
        services: ServiceProvider,
        _state: StateManager,
        db_tx: &mut DatabaseTransaction<'_>,
    ) -> Result<(), Error> {
        info!(
            collector = payload.kind.collector.to_string(),
            "Received a new FeeCollected event",
        );

        Ok(())
    }
}