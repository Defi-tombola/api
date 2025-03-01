use std::str::FromStr;

use crate::{
    events::WinnerPaid,
    handler::{Handler, HandlerPayload},
    state::StateManager,
};
use async_trait::async_trait;
use chrono::Utc;
use entity::{draw::DrawStatus, prize::PrizeStatus};
use error_stack::{Report, Result};
use lib::error::Error;
use rust_decimal::Decimal;
use service::{account::store::AccountStore, chain::{provider::ChainProvider, traits::string::ToHexString}, draw::{store::DrawStore, types::{CreateDraw, UpdateDraw}, DrawService}, lottery::store::LotteryStore, prize::{store::PrizeStore, types::CreatePrize, PrizeService}, store::service::{DatabaseTransaction, StoreService}, ticket::store::TicketStore};
use service::services::ServiceProvider;
use tracing::{info, warn};

#[async_trait]
impl<Provider> Handler<WinnerPaid> for Provider
where
    Provider: ChainProvider,
{
    async fn handle(
        &self,
        payload: HandlerPayload<WinnerPaid>,
        services: ServiceProvider,
        _state: StateManager,
        db_tx: &mut DatabaseTransaction<'_>,
    ) -> Result<(), Error> {
        info!(
            lottery_id = payload.kind.lottery_id.to_string(),
            winner = payload.kind.winner.to_string(),
            "Received a new WinnerPaid event",
        );
        let winner_address = payload.kind.winner.to_hex_string();
        
        let user = match AccountStore::try_find_by_address(
            db_tx.as_mut(), 
            winner_address.clone(),
        ).await? {
            Some(user) => user,
            None => {
                warn!(
                    lottery_id = payload.kind.lottery_id.to_string(),
                    winner = payload.kind.winner.to_string(),
                    "Winner not found in the database",
                );
                return Ok(());
            }
        };
        
        let lottery_uid = payload.kind.lottery_id.to_hex_string();
        let lottery = LotteryStore::find_by_uid(db_tx.as_mut(), lottery_uid).await?;
        
        let store_service = services.get_service_unchecked::<StoreService>().await;
        let pool = store_service.read();
        
        let winning_ticket = match TicketStore::find_by_lottery_id_and_account_id(
            pool, lottery.id, user.id).await? {
                Some(ticket) => ticket,
                None => {
                    warn!(
                        lottery_id = payload.kind.lottery_id.to_string(),
                        winner = payload.kind.winner.to_string(),
                        "Winning ticket not found in the database",
                    );
                    return Ok(());
                }
            };
        
        let draw_service = services.get_service_unchecked::<DrawService>().await;
        
        let event_context = payload.get_context(self);
        let draw_dto = UpdateDraw {
            draw_date: Some(payload.triggered_at),
            status: Some(DrawStatus::Completed),
            winner: Some(winning_ticket.account_id),
            transaction_hash: Some(payload.transaction_hash.to_hex_string())
        };
        
        let draw = draw_service.mark_winner_as_drawn(lottery.id, draw_dto, Some(event_context), db_tx).await?;
        
        info!(
            lottery_id = lottery.uid,
            winner = winner_address,
            "Winner paid",
        );
        Ok(())
    }
}