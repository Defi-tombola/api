use async_graphql::InputObject;
use entity::prelude::LotteryStatus;
use service::lottery::store::LotteryFilter;

#[derive(InputObject)]
pub struct LotteryFilterInput {
    pub uid: Option<String>,
    pub featured: Option<bool>,
    pub status: Option<LotteryStatus>,
}

impl From<LotteryFilterInput> for LotteryFilter {
    fn from(value: LotteryFilterInput) -> Self {
        Self {
            featured: value.featured,
            status: value.status,
            uid: value.uid,
        }
    }
}