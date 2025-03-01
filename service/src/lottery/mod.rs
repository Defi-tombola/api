pub mod utils;
pub mod store;
pub mod types;

use std::{fs, path::Path, sync::Arc};

use chrono::Utc;
use entity::{draw::{DrawModel, DrawStatus}, prelude::LotteryModel, prize::PrizeStatus};
use lib::error::Error;
use rand::Rng;
use rust_decimal::Decimal;
use serenity::async_trait;
use error_stack::{Result, ResultExt};
use store::LotteryStore;
use types::{CreateLottery, UpdateLottery};
use uuid::Uuid;
use crate::{chain::types::EventContext, draw::{store::DrawStore, types::{CreateDraw, UpdateDraw}, DrawService}, prelude::{ServiceProvider, StoreService}, prize::{store::PrizeStore, types::{CreatePrize, UpdatePrize}, PrizeService}, services::ServiceFactory, store::service::DatabaseTransaction, transaction::{service::TransactionService, types::{CreateTransaction, TransactionSideEffect}}};

pub struct LotteryService {
   pub store: Arc<StoreService>,
   pub transaction_service: Arc<TransactionService>,
   pub draw_service: Arc<DrawService>,
   pub prize_service: Arc<PrizeService>,
}

impl LotteryService {
    pub fn new(store: Arc<StoreService>, transaction_service: Arc<TransactionService>, draw_service: Arc<DrawService>, prize_service: Arc<PrizeService>) -> Self {
        Self {
            store,
            transaction_service,
            draw_service,
            prize_service,
        }
    }
    
    pub async fn close_lottery(
        &self,
        lottery_id: Uuid,
        input: UpdateLottery,
        context: Option<EventContext>,
        db_tx: &mut DatabaseTransaction<'_>,
    ) -> Result<LotteryModel, Error> {
        let lottery = LotteryStore::update(db_tx.as_mut(), lottery_id, input).await?;
        
        Ok(lottery)
    }
    
   
    pub async fn create_lottery(
        &self,
        input: CreateLottery,
        context: Option<EventContext>,
        db_tx: &mut DatabaseTransaction<'_>,
    ) -> Result<LotteryModel, Error> {
        let lottery = LotteryStore::create(db_tx.as_mut(), input).await?;
        
        let draw_dto = CreateDraw {
            lottery_id: lottery.id,
            status: DrawStatus::Pending,
        };
        
        let draw = self.draw_service.create(draw_dto, context.clone(), db_tx).await?;
        
        let prize_dto = CreatePrize {
            lottery_id: lottery.id,
            prize_asset: lottery.ticket_asset,
            value: Decimal::ZERO,
            status: PrizeStatus::Active,
        };
        
        let prize = self.prize_service.create(prize_dto, context, db_tx).await?;
        
        Ok(lottery)
    }
}

#[async_trait]
impl ServiceFactory for LotteryService {
    async fn factory(services:ServiceProvider) -> Result<Self, Error> {
        let store = services.get_service_unchecked::<StoreService>().await;
        let draw_service = services.get_service_unchecked::<DrawService>().await;
        let prize_service = services.get_service_unchecked::<PrizeService>().await;
        let transaction_service = services.get_service_unchecked::<TransactionService>().await;
        
        Ok(Self {
            store,
            transaction_service,
            draw_service,
            prize_service,
        })
    }
}