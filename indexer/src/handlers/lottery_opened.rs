use crate::{
    events::{LotteryOpened},
    handler::{Handler, HandlerPayload},
    state::StateManager,
};
use async_trait::async_trait;
use chrono::Utc;
use entity::prelude::LotteryStatus;
use error_stack::{Report, Result};
use ethers::types::Address;
use lib::error::Error;
use service::{asset::store::AssetStore, chain::utils::get_lottery_provider::get_lottery_data, lottery::{store::LotteryStore, types::CreateLottery, utils::generate_random_lottery_name, LotteryService}, prelude::ServiceProvider, store::service::StoreService};
use service::{
    chain::{provider::ChainProvider, traits::string::ToHexString},
    store::service::DatabaseTransaction,
};
use tracing::{info, warn};
use rust_decimal::{prelude::FromPrimitive, Decimal};

#[async_trait]
impl<Provider> Handler<LotteryOpened> for Provider
where
    Provider: ChainProvider,
{
    async fn handle(
        &self,
        payload: HandlerPayload<LotteryOpened>,
        services: ServiceProvider,
        _state: StateManager,
        db_tx: &mut DatabaseTransaction<'_>,
    ) -> Result<(), Error> {
        info!(
            lottery_id = payload.kind.lottery_id.to_string(),
            "Received a new LotteryOpened event",
        );
        
        let config = self.get_config();
        let lottery_provider_address = config.contracts.get("provider").unwrap();
        
        let lottery_data = get_lottery_data(self, *lottery_provider_address, payload.kind.lottery_id).await?;
        let token_address = lottery_data.entrance_token_address;
        
        let asset = AssetStore::find_by_address(db_tx.as_mut(), token_address.to_hex_string()).await?;
        
        let start_date = payload.triggered_at;
        let end_date = start_date + chrono::Duration::days(1);
        
        let ticket_price = payload.kind.ticket_price.as_u128();
        let ticket_fee = payload.kind.fee_amount_per_ticket.as_u128();
        let lottery_service = services.get_service_unchecked::<LotteryService>().await;
        
        let lottery_name = generate_random_lottery_name().unwrap_or(String::from("Mega Jackpot"));
        
        let dto = CreateLottery {
            name: lottery_name,
            uid: payload.kind.lottery_id.to_hex_string(),
            start_date,
            end_date,
            ticket_price: Decimal::from_u128(ticket_price).unwrap(),
            fee_ticket_amount: Decimal::from_u128(ticket_fee).unwrap(),
            ticket_asset: asset.id,
            max_tickets: Some(payload.kind.max_tickets as i32),
            status: LotteryStatus::Ongoing
        };
        
        let context = payload.get_context(self);
        
        let lottery = lottery_service.create_lottery(dto, Some(context), db_tx).await?;

        info!("Lottery created: {:?}", lottery);
        
        Ok(())
    }
}
