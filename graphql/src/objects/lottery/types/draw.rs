use async_graphql::{Context, Enum, Object};
use chrono::{Days, Utc};
use entity::draw::{DrawModel, DrawStatus};
use service::account::store::AccountStore;
use sqlx::types::Decimal;

use entity::prelude::{AssetModel, LotteryModel};
use service::{
    prelude::{ConfigService, StoreService},
    services::ServiceProvider,
};

use crate::objects::account::types::AccountType;

pub struct DrawType(pub DrawModel);

impl From<DrawModel> for DrawType {
    fn from(value: DrawModel) -> Self {
        DrawType(value)
    }
}

#[Object]
impl DrawType {
    async fn id(&self) -> String {
        format!("{:#x}", self.0.id)
    }
    
    async fn lottery_id(&self) -> String {
        format!("{:#x}", self.0.lottery_id)
    }

    async fn winner(&self, ctx: &Context<'_>) -> Option<AccountType> {
        if let Some(winner) = self.0.winner {
            let services = ctx.data_unchecked::<ServiceProvider>();
            let store_service = services.get_service_unchecked::<StoreService>().await;
            let pool = store_service.read();
            
            match AccountStore::try_find_by_id(pool, winner).await.ok()? {
                Some(account) => Some(account.into()),
                None => None,
            }
        } else {
            None
        }
    }

    async fn transaction_hash(&self) -> Option<String> {
        self.0.transaction_hash.clone()
    }
    
    async fn draw_date(&self) -> Option<String> {
        self.0.draw_date
            .map(|date| date.to_rfc3339())
    }

    async fn status(&self) -> DrawStatus {
        self.0.status
    }

    async fn updated_at(&self) -> String {
        self.0.updated_at.to_rfc3339()
    }
}
