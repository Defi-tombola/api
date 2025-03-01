pub mod store;
pub mod types;
use chrono::Utc;
use entity::{draw::DrawModel, prelude::LotteryModel, prize::PrizeModel, ticket::TicketModel};
use lib::error::Error;
use serenity::async_trait;
use error_stack::{Result, ResultExt};
use store::{PrizeStore};
use types::{CreatePrize,};
use crate::{chain::types::EventContext, prelude::{ServiceProvider, StoreService}, services::ServiceFactory, store::service::DatabaseTransaction, transaction::{service::TransactionService, types::{CreateTransaction, TransactionSideEffect}}};

use std::sync::Arc;

pub struct PrizeService {
   pub store: Arc<StoreService>,
   pub transaction_service: Arc<TransactionService>,
}

impl PrizeService {
    pub fn new(store: Arc<StoreService>, transaction_service: Arc<TransactionService>) -> Self {
        Self {
            store,
            transaction_service,
        }
    }
    
    pub async fn create(
        &self,
        input: CreatePrize,
        context: Option<EventContext>,
        db_tx: &mut DatabaseTransaction<'_>,
    ) -> Result<PrizeModel, Error> {
        let prize = PrizeStore::create(db_tx.as_mut(), input).await?;
        
        Ok(prize)
    }
    
}

#[async_trait]
impl ServiceFactory for PrizeService {
    async fn factory(services:ServiceProvider) -> Result<Self, Error> {
        let store = services.get_service_unchecked::<StoreService>().await;
        
        Ok(Self {
            store,
            transaction_service: services.get_service_unchecked::<TransactionService>().await,
        })
    }
}