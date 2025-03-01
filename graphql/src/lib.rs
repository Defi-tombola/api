pub mod guards;
pub mod helpers;
pub mod ide;
pub mod loaders;
pub mod objects;
pub mod routes;
pub mod schema;
pub mod server;
pub mod validators;

use crate::server::Server;
use error_stack::Result;
use futures::future;
use lib::error::Error;
use service::{
     cache::service::CacheService, common::shutdown::ShutdownFlag, config::service::ConfigService, message_broker::MessageBrokerService, services::ServiceProvider, store::service::StoreService
};
use tracing::{info, warn, Instrument};

pub async fn start(config: ConfigService) -> Result<(), Error> {
    info!(version = %env!("CARGO_PKG_VERSION"), "Starting GraphQL");

    // Init services and manually inject config service
    let services = ServiceProvider::new();
    let config = services.add_service(config).await;

    services.warm_up::<StoreService>().await;
    services.warm_up::<CacheService>().await;
    services.warm_up::<MessageBrokerService>().await;

    info!("Service is listening at {}", config.graphql.listen);
    info!("GraphQL endpoint exposed at {}", config.graphql.endpoint);
    info!(
        "GraphQL subscription endpoint exposed at {}",
        config.graphql.subscription_endpoint
    );
    info!(
        "Playground endpoint available at {}",
        config.graphql.endpoint
    );
    info!(
        "Health-check endpoint available at {}/health",
        config.graphql.endpoint
    );

    let server = Server::new(services);
    let server = server.start().in_current_span();
    
    if let Err(err) = server.await {
        warn!("Error while server task: {}", err);
    }

    Ok(())
}
