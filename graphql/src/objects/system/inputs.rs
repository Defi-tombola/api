use async_graphql::InputObject;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, InputObject)]
pub struct SimulateTransactionInput {
    pub network_id: u32,
    pub from: String,
    pub to: String,
    pub input: String,
    pub gas: u32,
    pub gas_price: u32,
    pub value: String,
}
