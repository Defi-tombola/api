use async_graphql::{Context, Object};
use chrono::{Days, Utc};
use service::account::store::AccountStore;
use service::lottery::store::LotteryStore;
use sqlx::types::Decimal;

use entity::prelude::{AssetModel, TicketModel};
use service::asset::store::AssetStore;
use service::{
    prelude::{ConfigService, StoreService},
    services::ServiceProvider,
};

use crate::objects::account::types::AccountType;
use crate::objects::asset::types::AssetType;

pub struct TicketType(TicketModel);

impl From<TicketModel> for TicketType {
    fn from(value: TicketModel) -> Self {
        TicketType(value)
    }
}

#[Object]
impl TicketType {
    async fn id(&self) -> String {
        format!("{:#x}", self.0.id)
    }
    
    async fn lottery_id (&self) -> String {
        format!("{:#x}", self.0.lottery_id)
    }
    
    async fn lottery_uid(&self, ctx: &Context<'_>) -> async_graphql::Result<String> {
        let services = ctx.data_unchecked::<ServiceProvider>();
            let store_service = services.get_service_unchecked::<StoreService>().await;
            let pool = store_service.read();
            
            match LotteryStore::find_by_id(pool, self.0.lottery_id).await {
                Ok(lottery) => Ok(lottery.uid),
                Err(_) => Err(async_graphql::Error::new("Unable to find lottery")),
            }
    }
    
    async fn asset(&self, ctx: &Context<'_>) -> async_graphql::Result<AssetType> {
        let services = ctx.data_unchecked::<ServiceProvider>();
        let store_service = services.get_service_unchecked::<StoreService>().await;
        let pool = store_service.read();
        
        let asset = match AssetStore::find_by_id(pool, self.0.ticket_asset).await {
            Ok(asset) => Ok(asset),
            Err(_) => Err(async_graphql::Error::new("Unable to find asset"))
        }?;
        
        Ok(asset.into())
    }
    
    async fn account(&self, ctx: &Context<'_>) -> AccountType {
        let services = ctx.data_unchecked::<ServiceProvider>();
        let store_service = services.get_service_unchecked::<StoreService>().await;
        let pool = store_service.read();
        
        let account = AccountStore::find_by_id(pool, self.0.account_id).await.unwrap();
        account.into()
    }
    
    /// Represent the number of tickets the user has bought on this transaction
    async fn n_tickets(&self) -> u32 {
        self.0.amount as u32
    }
    
    /// Represent the price of each ticket
    async fn individual_ticket_price(&self) -> Decimal {
        self.0.ticket_price.into()
    }
    
    /// Represent the total price of all the tickets bought
    async fn total_price(&self) -> Decimal {
        self.0.ticket_price.saturating_mul(Decimal::from(self.0.amount as u32))
    }

    async fn purchased_at(&self) -> String {
        self.0.purchased_at.to_rfc3339()
    }
    
    async fn transaction_hash(&self) -> String {
        self.0.transaction_hash.clone()
    }

    async fn updated_at(&self) -> String {
        self.0.updated_at.to_rfc3339()
    }
    
    async fn created_at(&self) -> String {
        self.0.created_at.to_rfc3339()
    }
}
