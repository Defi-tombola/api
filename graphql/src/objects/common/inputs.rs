use async_graphql::{Enum, InputObject};
use serde::Serialize;

#[derive(Clone, Debug, Serialize, InputObject, Default)]
pub struct PageInput {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}