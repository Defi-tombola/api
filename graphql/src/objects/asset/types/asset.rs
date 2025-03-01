use async_graphql::{Context, Object};
use chrono::{Days, Utc};
use sqlx::types::Decimal;

use entity::prelude::AssetModel;
use service::asset::store::AssetStore;
use service::{
    prelude::{ConfigService, StoreService},
    services::ServiceProvider,
};
use tracing::warn;

pub struct AssetType(AssetModel);

impl From<AssetModel> for AssetType {
    fn from(value: AssetModel) -> Self {
        AssetType(value)
    }
}

#[Object]
impl AssetType {
    async fn id(&self) -> String {
        format!("{:#x}", self.0.id)
    }

    async fn name(&self) -> &str {
        &self.0.name
    }

    async fn address(&self) -> &str {
        &self.0.address
    }

    async fn symbol(&self) -> &str {
        &self.0.symbol
    }

    async fn decimals(&self) -> Option<i16> {
        self.0.decimals
    }

    async fn deprecated(&self) -> bool {
        self.0.deprecated
    }

    #[graphql(name = "type")]
    async fn asset_type(&self) -> String {
        self.0.asset_type.to_string()
    }

    async fn created_at(&self) -> String {
        self.0.created_at.to_rfc3339()
    }
}
