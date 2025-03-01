
pub mod store;
pub mod types;
use std::sync::Arc;

use chrono::Utc;
use colorful::core::StrMarker;
use entity::{draw::DrawModel, prelude::LotteryModel, ticket::TicketModel};
use lib::error::Error;
use rust_decimal::Decimal;
use serenity::async_trait;
use error_stack::{Report, Result};
use store::TicketStore;
use tracing::{error, info};
use types::CreateTicket;
use crate::{chain::types::EventContext, lottery::store::LotteryStore, message_broker::MessageBrokerService, prelude::{ServiceProvider, StoreService}, prize::{store::PrizeStore, types::UpdatePrize}, services::ServiceFactory, store::service::DatabaseTransaction, transaction::{service::TransactionService, types::{CreateTransaction, TransactionSideEffect}}};

pub struct TicketService {
   pub store: Arc<StoreService>,
   pub transaction_service: Arc<TransactionService>,
   pub message_broker: Arc<MessageBrokerService>,
}

impl TicketService {
    pub fn new(store: Arc<StoreService>, transaction_service: Arc<TransactionService>, message_broker: Arc<MessageBrokerService>) -> Self {
        Self {
            store,
            transaction_service,
            message_broker
        }
    }
    
    pub async fn buy_tickets(
        &self,
        input: CreateTicket,
        context: Option<EventContext>,
        db_tx: &mut DatabaseTransaction<'_>,
    ) -> Result<TicketModel, Error> {
        let tickets = TicketStore::create(db_tx.as_mut(), input.clone()).await?;
        
        // We should now update prize pool
        let lottery = LotteryStore::find_by_id(db_tx.as_mut(), tickets.lottery_id).await?;
        
        let prize_pool = PrizeStore::find_by_lottery_id(db_tx.as_mut(), lottery.id).await?;
        
        let entrance_fee = lottery.ticket_price;
        let tickets_value = Decimal::from(input.amount as i64).checked_mul(entrance_fee);
        
        if tickets_value.is_none() {
            return Err(Report::new(Error::TicketServiceInvalidFee));
        }
        
        let ticket_value  = tickets_value.unwrap();
        let total_prize_pool_value = prize_pool.value.saturating_add(ticket_value);
        
        let dto = UpdatePrize {
            value: Some(total_prize_pool_value),
            ..Default::default()
        };
        
        let prize = PrizeStore::update(db_tx.as_mut(), prize_pool.id, dto).await?;
        
        if let Err(e)  = self.message_broker.send("ticket_bought".to_string(), tickets.clone()).await {
            error!("Failed to send ticket bought event: {e:?}");
        }
    
        Ok(tickets)
    }
    
}

#[async_trait]
impl ServiceFactory for TicketService {
    async fn factory(services:ServiceProvider) -> Result<Self, Error> {
        let store = services.get_service_unchecked::<StoreService>().await;
        let message_broker = services.get_service_unchecked::<MessageBrokerService>().await;
        
        Ok(Self {
            store,
            transaction_service: services.get_service_unchecked::<TransactionService>().await,
            message_broker,
        })
    }
}