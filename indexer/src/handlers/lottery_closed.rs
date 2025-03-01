use crate::{
    events::LotteryClosed,
    handler::{Handler, HandlerPayload},
    state::StateManager,
};
use async_trait::async_trait;
use entity::prelude::LotteryStatus;
use error_stack::{Report, Result};
use lib::error::Error;
use service::{chain::{provider::ChainProvider, traits::string::ToHexString}, lottery::{store::LotteryStore, types::UpdateLottery, LotteryService}, store::service::{DatabaseTransaction, StoreService}};
use service::services::ServiceProvider;
use tracing::{info, warn};

#[async_trait]
impl<Provider> Handler<LotteryClosed> for Provider
where
    Provider: ChainProvider,
{
    async fn handle(
        &self,
        payload: HandlerPayload<LotteryClosed>,
        services: ServiceProvider,
        _state: StateManager,
        db_tx: &mut DatabaseTransaction<'_>,
    ) -> Result<(), Error> {
        info!(
            lottery_id = payload.kind.lottery_id.to_hex_string(),
            "Received a new LotteryClosed event",
        );
        
        let dto = UpdateLottery {
            status: Some(LotteryStatus::Completed),
            ..Default::default()
        };
        
        let lottery = LotteryStore::find_by_uid(
            db_tx.as_mut(), 
            payload.kind.lottery_id.to_hex_string(),
        ).await?;
        
        let lottery_service = services.get_service_unchecked::<LotteryService>().await;
        let context = payload.get_context(self);
        
        lottery_service.close_lottery(lottery.id, dto, Some(context), db_tx).await?;
        
        info!(
            lottery_id = lottery.uid,
            "Lottery closed",
        );
        
        Ok(())
    }
}