use contract::LotteryProvider;
use ethers::abi::Address;
use lib::error::Error;
use serenity::async_trait;
use error_stack::{Result, ResultExt};

use super::{provider::ChainProvider, ChainClient};

#[async_trait]
pub trait ContractProvider<Contract>
where
    Self: ChainProvider,
{
    async fn get_contract(&self, address: Option<Address>) -> Result<Contract, Error>;
}
