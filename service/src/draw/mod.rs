use std::sync::Arc;

use chrono::Utc;
use entity::{draw::DrawModel, prize::PrizeStatus};
use lib::error::Error;
use serenity::async_trait;
use error_stack::{Result, ResultExt};
use store::DrawStore;
use types::{CreateDraw, UpdateDraw};
use uuid::Uuid;
use crate::{chain::types::EventContext, prelude::{ServiceProvider, StoreService}, prize::{store::PrizeStore, types::UpdatePrize}, services::ServiceFactory, store::service::DatabaseTransaction, transaction::{service::TransactionService, types::{CreateTransaction, TransactionSideEffect}}};

pub mod store;
pub mod types;

pub struct DrawService {
   pub store: Arc<StoreService>,
   pub transaction_service: Arc<TransactionService>,
}

impl DrawService {
    pub fn new(store: Arc<StoreService>, transaction_service: Arc<TransactionService>) -> Self {
        Self {
            store,
            transaction_service,
        }
    }
    
    pub async fn create(
        &self,
        input: CreateDraw,
        context: Option<EventContext>,
        db_tx: &mut DatabaseTransaction<'_>,
    ) -> Result<DrawModel, Error> {
        let draw = DrawStore::create(db_tx.as_mut(), input).await?;
        
        Ok(draw)
    }
    
    pub async fn mark_winner_as_drawn(
        &self,
        lottery_id: Uuid,
        input: UpdateDraw,
        context: Option<EventContext>,
        db_tx: &mut DatabaseTransaction<'_>,
    ) -> Result<DrawModel, Error> {
        let prize_dto = UpdatePrize {
            status: Some(PrizeStatus::Distributed),
            ..Default::default()
        };
        
        let prize = PrizeStore::find_by_lottery_id(db_tx.as_mut(), lottery_id.clone()).await?;
        let draw = DrawStore::find_by_lottery_id(db_tx.as_mut(), lottery_id.clone()).await?;
        
        let prize = PrizeStore::update(db_tx.as_mut(), prize.id, prize_dto).await?;
        let draw = DrawStore::update(db_tx.as_mut(), draw.id, input).await?;
        
        Ok(draw)
    }
}

#[async_trait]
impl ServiceFactory for DrawService {
    async fn factory(services:ServiceProvider) -> Result<Self, Error> {
        let store = services.get_service_unchecked::<StoreService>().await;
        
        Ok(Self {
            store,
            transaction_service: services.get_service_unchecked::<TransactionService>().await,
        })
    }
}