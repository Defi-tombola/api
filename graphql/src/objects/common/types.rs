use async_graphql::{OutputType, SimpleObject};
use chrono::NaiveDateTime;
use service::{common::types::ChartDataset as ChartDatasetBase,};

#[derive(SimpleObject)]
pub struct Page<T: Sync + Send + OutputType> {
    pub num_pages: u64,
    pub num_items: u64,
    pub items: Vec<T>,
}
