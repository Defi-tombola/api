use ethers::prelude::EthEvent;
use ethers::types::{Address, Bytes, H256, U128, U256, U64};

// #[ethevent(
//     abi = "LotteryOpened(bytes32,uint128,uint128,uint32)"
// )]
#[derive(Clone, Debug, EthEvent)]
pub struct LotteryOpened {
    #[ethevent(indexed)]
    pub lottery_id: H256,
    pub ticket_price: U128,
    pub fee_amount_per_ticket: U128,
    pub max_tickets: u32
}

#[derive(Clone, Debug, EthEvent)]
pub struct LotteryClosed {
    #[ethevent(indexed)]
    pub lottery_id: H256,
}

#[derive(Clone, Debug, EthEvent)]
pub struct TicketBought {
    #[ethevent(indexed)]
    pub lottery_id: H256,
    #[ethevent(indexed)]
    pub buyer: Address,
    pub tickets: u32,
}

#[derive(Clone, Debug, EthEvent)]
pub struct WinnerPaid {
    #[ethevent(indexed)]
    pub lottery_id: H256,
    #[ethevent(indexed)]
    pub winner: Address,
    pub amount: U256,
}

#[derive(Clone, Debug, EthEvent)]
pub struct LotteryNumberGenerated {
    #[ethevent(indexed)]
    pub requester: Address,
    #[ethevent(name = "rNumber")]
    pub r_number: U256,
}

#[derive(Clone, Debug, EthEvent)]
pub struct LotteryCanceled {
    #[ethevent(indexed)]
    pub lottery_id: H256,
}

#[derive(Clone, Debug, EthEvent)]
pub struct FeeCollected {
    #[ethevent(indexed)]
    pub collector: Address,
    pub amount: U256,
}

#[derive(Clone, Debug, EthEvent)]
pub struct LotteryProviderUpdated {
    #[ethevent(indexed)]
    pub lottery_provider: Address,
}