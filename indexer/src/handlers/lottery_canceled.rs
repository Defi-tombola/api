use crate::{
    events::LotteryCanceled,
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
impl<Provider> Handler<LotteryCanceled> for Provider
where
    Provider: ChainProvider,
{
    async fn handle(
        &self,
        payload: HandlerPayload<LotteryCanceled>,
        services: ServiceProvider,
        _state: StateManager,
        db_tx: &mut DatabaseTransaction<'_>,
    ) -> Result<(), Error> {
        info!(
            lottery_id = payload.kind.lottery_id.to_string(),
            "Received a new LotteryCanceled event",
        );
        
        // TODO

        Ok(())
    }
}