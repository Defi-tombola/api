pub mod account;
pub mod asset;
pub mod common;
pub mod image;
pub mod system;
pub mod twitter;
pub mod lottery;

use async_graphql::{MergedObject, MergedSubscription};
use lottery::{tickets::{TicketQuery, TicketSubscription}, LotteryQuery};

use self::{
    account::{AccountMutation, AccountQuery, AccountSubscription},
    asset::{AssetQuery, AssetSubscription},
    // image::ImageMutation,
    twitter::{TwitterMutation, TwitterQuery},
};
use crate::helpers::jwt::Claims;

#[derive(MergedObject, Default)]
pub struct Query(
    AccountQuery,
    AssetQuery,
    TwitterQuery,
    LotteryQuery,
    TicketQuery
);

#[derive(MergedObject, Default)]
pub struct Mutation(
    TwitterMutation,
    AccountMutation,
    // ImageMutation,
);

#[derive(MergedSubscription, Default)]
pub struct Subscription(
    TicketSubscription
);

pub struct GQLJWTData {
    pub claims: Option<Claims>,
}
