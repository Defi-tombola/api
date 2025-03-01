use async_graphql::{Context, Object};
use chrono::{Days, Utc};
use entity::prize::PrizeStatus;
use sqlx::types::Decimal;

use entity::prelude::{AssetModel, PrizeModel};
use service::asset::store::AssetStore;
use service::{
    prelude::{ConfigService, StoreService},
    services::ServiceProvider,
};

use crate::objects::asset::types::AssetType;

pub struct PrizeType(PrizeModel);

impl From<PrizeModel> for PrizeType {
    fn from(value: PrizeModel) -> Self {
        PrizeType(value)
    }
}

#[Object]
impl PrizeType {
    async fn id(&self) -> String {
        format!("{:#x}", self.0.id)
    }
    
    async fn lottery_id (&self) -> String {
        format!("{:#x}", self.0.lottery_id)
    }
    
    async fn prize_asset(&self, ctx: &Context<'_>) -> AssetType {
        let services = ctx.data_unchecked::<ServiceProvider>();
        let store_service = services.get_service_unchecked::<StoreService>().await;
        let pool = store_service.read();
        
        let asset = AssetStore::find_by_id(pool, self.0.prize_asset).await.unwrap();
        asset.into()
    }
    
    async fn total_prize_pool(&self) -> Decimal {
        self.0.value
    }

    async fn status(&self) -> PrizeStatus {
        self.0.status
    }

    async fn updated_at(&self) -> String {
        self.0.updated_at.to_rfc3339()
    }
    
    async fn created_at(&self) -> String {
        self.0.created_at.to_rfc3339()
    }
}
