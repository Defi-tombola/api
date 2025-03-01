use std::str::FromStr;

use crate::objects::{Mutation, Query, Subscription as SubscriptionRoot};
use crate::{
    helpers::jwt::JWT,
    loaders::{
        account::AccountLoader, asset::AssetLoader
    },
};
use async_graphql::extensions::{Tracing, OpenTelemetry};
use async_graphql::EmptySubscription;
use async_graphql::{dataloader::DataLoader, Schema};
use service::prelude::StoreService;
use service::services::ServiceProvider;
use service::telemetry::get_tracer;
use uuid::Uuid;

pub type ServiceSchema = Schema<Query, Mutation, SubscriptionRoot>;

#[derive(Clone)]
pub struct GQLGlobalData {
    pub services: ServiceProvider,
    pub jwt: JWT,
}

#[derive(Clone)]
pub struct OrganizationId(pub Uuid);

impl From<String> for OrganizationId {
    fn from(id: String) -> Self {
        Self(Uuid::from_str(&id).unwrap())
    }
}

#[buildstructor::buildstructor]
impl GQLGlobalData {
    #[builder(entry = "builder", exit = "build", visibility = "pub")]
    fn builder_new(services: ServiceProvider, jwt: JWT) -> Self {
        Self { services, jwt }
    }
}

/// Init GraphQL schema
pub async fn new(ctx: GQLGlobalData) -> ServiceSchema {
    let store = ctx.services.get_service_unchecked::<StoreService>().await;
    
    let global_tracer = get_tracer("graphql-server".to_string());
    
    let opentelemetry_extension = OpenTelemetry::new(global_tracer);
    
    Schema::build(
        Query::default(),
        Mutation::default(),
        SubscriptionRoot::default(),
    )
    .data(ctx.jwt)
    .data(ctx.services)
    .data(DataLoader::new(
        AccountLoader::new(store.clone()),
        tokio::spawn,
    ))
    .data(DataLoader::new(
        AssetLoader::new(store.clone()),
        tokio::spawn,
    ))
    .extension(Tracing)
    .extension(opentelemetry_extension)
    .finish()
}
