use crate::{
    events::TicketBought,
    handler::{Handler, HandlerPayload},
    state::StateManager,
};
use async_trait::async_trait;
use chrono::Utc;
use error_stack::{Report, Result};
use lib::error::Error;
use service::{account::{store::AccountStore, types::CreateAccount, AccountService}, chain::{provider::ChainProvider, traits::string::ToHexString}, lottery::store::LotteryStore, store::service::{DatabaseTransaction, StoreService}, ticket::{store::TicketStore, types::{CreateTicket, UpdateTicket}, TicketService}};
use service::services::ServiceProvider;
use tracing::{info, warn};

#[async_trait]
impl<Provider> Handler<TicketBought> for Provider
where
    Provider: ChainProvider,
{
    async fn handle(
        &self,
        payload: HandlerPayload<TicketBought>,
        services: ServiceProvider,
        _state: StateManager,
        db_tx: &mut DatabaseTransaction<'_>,
    ) -> Result<(), Error> {
        info!(
            lottery_id = payload.kind.lottery_id.to_string(),
            buyer = payload.kind.buyer.to_string(),
            "Received a new TicketBought event",
        );
        let account_service = services.get_service_unchecked::<AccountService>().await;
        
        let user = match AccountStore::try_find_by_address(db_tx.as_mut(), payload.kind.buyer.to_hex_string().clone()).await? {
            Some(user) => user,
            None => {
                let dto = CreateAccount {
                    address: payload.kind.buyer.to_hex_string(),
                    created_at: payload.triggered_at
                };
                
                let user = account_service.create_if_no_exists(dto, db_tx).await?;
                
                user
            }
        };
        
        let lottery_uid = payload.kind.lottery_id.to_hex_string();
        let lottery = match LotteryStore::try_find_by_uid(db_tx.as_mut(), lottery_uid.clone()).await? {
            Some(lottery) => lottery,
            None => {
                warn!(
                    lottery_uid = lottery_uid.to_string(),
                    "Could not find lottery with UID",
                );
                return Ok(());
            }
        };
        
        let account_id = user.id;
        
        // Create new ticket
        let dto = CreateTicket {
           purchased_at: payload.triggered_at,
           account_id,
           ticket_asset: lottery.ticket_asset,
           ticket_price: lottery.ticket_price,
           lottery_id: lottery.id,
           amount: payload.kind.tickets as i32,
           transaction_hash: payload.transaction_hash.to_hex_string()
        };
        
        let ticket_service = services.get_service_unchecked::<TicketService>().await;
        let context = payload.get_context(self);
        let ticket = ticket_service.buy_tickets(dto, Some(context), db_tx).await?;
        
        info!(
            lottery_id = lottery.id.to_string(),
            buyer = payload.kind.buyer.to_string(),
            ticket_id = ticket.id.to_string(),
            "Created new ticket",
        );
        
        Ok(())
    }
}