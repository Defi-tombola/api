use async_graphql::{Context, Object};
use inputs::LotteryFilterInput;
use service::{draw::store::DrawStore, lottery::store::LotteryStore, prelude::{ServiceProvider, StoreService}};
use tracing::warn;
use types::{DrawType, LotteryType};

pub mod types;
pub mod inputs;
pub mod tickets;

#[derive(Default)]
pub struct LotteryQuery;

#[Object]
impl LotteryQuery {
    /// Get list of all supported assets
    async fn lotteries(&self, ctx: &Context<'_>, input: LotteryFilterInput) -> async_graphql::Result<Vec<LotteryType>> {
        let services = ctx.data_unchecked::<ServiceProvider>();
        let store_service = services.get_service_unchecked::<StoreService>().await;

        let pool = store_service.read();
        let lotteries = LotteryStore::find_by_filter(pool, input.into()).await.map_err(|e| {
            warn!("Failed to lotteries: {e:?}");
            async_graphql::Error::from("Internal error")
        })?;
        
        Ok(lotteries.into_iter().map(Into::into).collect())
    }
    
    async fn lottery(&self, ctx: &Context<'_>, uid: String) -> async_graphql::Result<LotteryType> {
        let services = ctx.data_unchecked::<ServiceProvider>();
        let store_service = services.get_service_unchecked::<StoreService>().await;

        let pool = store_service.read();
        let lottery = LotteryStore::find_by_uid(pool, uid).await.map_err(|e| {
            warn!("Failed to lottery: {e:?}");
            async_graphql::Error::from("Internal error")
        })?;
        
        Ok(lottery.into())
    }
    
    async fn verify(&self, ctx: &Context<'_>, tx_hash: String) -> async_graphql::Result<Option<DrawType>> {
        let services = ctx.data_unchecked::<ServiceProvider>();
        let store_service = services.get_service_unchecked::<StoreService>().await;

        let pool = store_service.read();
        let draw = DrawStore::try_find_by_transaction_hash(pool, tx_hash).await.map_err(|e| {
            warn!("Failed to fetch draw: {e:?}");
            async_graphql::Error::from("Internal error")
        })?;
        
        if let Some(draw) = draw {
            Ok(Some(draw.into()))
        } else {
            Ok(None)
        }
    }
}


