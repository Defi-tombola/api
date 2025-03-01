use async_graphql::{Context, Object};
use chrono::{Days, Utc};
use entity::draw::DrawStatus;
use service::draw::store::DrawStore;
use service::prize::store::PrizeStore;
use service::ticket::store::TicketStore;
use sqlx::types::Decimal;

use entity::prelude::{AssetModel, LotteryModel, LotteryStatus};
use service::asset::store::AssetStore;
use service::{
    prelude::{ConfigService, StoreService},
    services::ServiceProvider,
};

use crate::objects::asset::types::AssetType;

use super::{DrawType, PrizeType, TicketType};

pub struct LotteryType(LotteryModel);

impl From<LotteryModel> for LotteryType {
    fn from(value: LotteryModel) -> Self {
        LotteryType(value)
    }
}

#[Object]
impl LotteryType {
    async fn id(&self) -> String {
        format!("{:#x}", self.0.id)
    }
    
    async fn tickets(&self, ctx: &Context<'_>) -> Vec<TicketType> {
        let services = ctx.data_unchecked::<ServiceProvider>();
        let store_service = services.get_service_unchecked::<StoreService>().await;
        let pool = store_service.read();
        
        let tickets = TicketStore::find_last_bought_tickets_by_lottery_id(pool, self.0.id).await.unwrap();
        tickets.into_iter().map(|ticket| ticket.into()).collect()
    }
    
    async fn draw(&self, ctx: &Context<'_>) -> Option<DrawType> {
        let services = ctx.data_unchecked::<ServiceProvider>();
        let store_service = services.get_service_unchecked::<StoreService>().await;
        let pool = store_service.read();
        
        let draw = DrawStore::find_by_lottery_id(pool, self.0.id).await;
        
        if draw.is_err() {
            return None;
        }
        
        let draw = draw.unwrap();
        if draw.status.eq(&DrawStatus::Pending) {
            return None;
        }
        
        Some(draw.into())
    }
    
    async fn prize(&self, ctx: &Context<'_>) -> async_graphql::Result<PrizeType> {
        let services = ctx.data_unchecked::<ServiceProvider>();
        let store_service = services.get_service_unchecked::<StoreService>().await;
        let pool = store_service.read();
        
        match PrizeStore::find_by_lottery_id(pool, self.0.id).await {
            Ok(prize) => Ok(prize.into()),
            Err(_) => Err(async_graphql::Error::new("No prize found"))
        }
    }
    
    async fn uid(&self) -> String {
        self.0.uid.clone()
    }

    async fn name(&self) -> &String {
        &self.0.name
    }

    async fn start_date(&self) -> String {
        self.0.start_date.to_rfc3339()
    }

    async fn end_date(&self) -> String {
        self.0.end_date.to_rfc3339()
    }

    async fn ticket_price(&self) -> Decimal {
        self.0.ticket_price
    }

    async fn fee_ticket_amount(&self) -> Decimal {
        self.0.fee_ticket_amount
    }

    async fn ticket_asset(&self, ctx: &Context<'_>) -> Option<AssetType> {
        let services = ctx.data_unchecked::<ServiceProvider>();
        let store_service = services.get_service_unchecked::<StoreService>().await;
        let pool = store_service.read();
        
        let asset = AssetStore::find_by_id(pool, self.0.ticket_asset).await.ok()?;
        Some(asset.into())
    }
    
    async fn max_tickets(&self) -> i32 {
        self.0.max_tickets.unwrap_or(0 as i32)
    }

    async fn status(&self) -> LotteryStatus {
        self.0.status
    }

    async fn updated_at(&self) -> String {
        self.0.updated_at.to_rfc3339()
    }
    
    async fn created_at(&self) -> String {
        self.0.created_at.to_rfc3339()
    }
}
