use contract::LotteryProvider as LotteryProviderContract;
use ethers::{abi::Address, types::H256};
use lib::error::Error;
use error_stack::{Result, ResultExt};

use crate::chain::provider::ChainProvider;

/// Get the parent vault for the given address
pub async fn get_lottery_data<Provider: ChainProvider>(
    provider: &Provider,
    address: Address,
    lottery_id: H256
) -> Result<LotteryChainData, Error> {
    let client = provider.get_client()?;
    
    let contract = LotteryProviderContract::new(address, client);
    
    let lottery = contract.lotteries(lottery_id.to_fixed_bytes())
        .call()
        .await
        .change_context(Error::ContractQuery)?;
    
    let entrance_token_address = lottery.0;
    let fee_amount_per_time = lottery.1;
    let ticket_price = lottery.2;
    let max_tickets = lottery.3;
    let sold_tickets = lottery.4;
    let is_active = lottery.5;
    let winner = lottery.6;
    
    Ok(LotteryChainData { 
        entrance_token_address,
        fee_amount_per_time,
        ticket_price,
        max_tickets,
        sold_tickets,
        is_active,
        winner,
    })
}

pub struct LotteryChainData {
   pub entrance_token_address: Address,
   pub fee_amount_per_time: u128,
   pub ticket_price: u128,
   pub max_tickets: u32,
   pub sold_tickets: u32,
   pub is_active: bool,
   pub winner: Address,
}