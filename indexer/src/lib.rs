mod chain;
mod events;
mod handler;
mod handlers;
mod state;
mod stream;

use crate::stream::{StreamProvider, StreamProviderResult};
use error_stack::Result;
use futures_util::future::try_join_all;
use lib::error::Error;
use service::cache::service::CacheService;
use service::chain::Chain;
use service::common::shutdown::{spawn_ctrl_c_listener, ShutdownFlag};
use service::config::service::{ChainConfig, ConfigService};
use service::services::ServiceProvider;
use service::store::service::StoreService;
use tracing::warn;
use tracing::{error, info};

/// Start the indexer with the provided configuration.
pub async fn start(
    config: ConfigService,
    chains: Option<Vec<String>>,
    with_tasks: bool,
) -> Result<(), Error> {
    info!(version = %env!("CARGO_PKG_VERSION"), "Starting indexer");

    // Init shutdown flag
    let shutdown = spawn_ctrl_c_listener();

    // Init services and manually inject config service
    let services = ServiceProvider::new();
    let config = services.add_service(config).await;

    services.warm_up::<StoreService>().await;
    services.warm_up::<CacheService>().await;

    // Start all chains or only selected ones
    let configs: Vec<ChainConfig> = config
        .chains
        .iter()
        .filter(|chain| {
            chains
                .as_ref()
                .map(|i| i.contains(&chain.name))
                .unwrap_or(true)
        })
        .cloned()
        .collect();

    if configs.is_empty() {
        warn!("No chains to start, check your config file or command line arguments");
        return Ok(());
    }

    if with_tasks {
        info!("Starting tasks");
    }

    let chain_tasks = start_chains(configs, services.clone(), shutdown.clone());

    match try_join_all(chain_tasks).await {
        Ok(results) => {
            for result in results {
                if let Err(e) = result {
                    error!(reason = ?e, "Critical error while running chain provider");
                }
            }
        }
        Err(e) => error!(reason = ?e, "Error while starting chains"),
    }

    Ok(())
}

/// Initialize chain for each provided config, and start them.
fn start_chains(
    config: Vec<ChainConfig>,
    services: ServiceProvider,
    shutdown: ShutdownFlag,
) -> Vec<StreamProviderResult> {
    config
        .into_iter()
        .map(|config| {
            let chain = Chain::from((config, services.clone()));
            chain.start(shutdown.clone())
        })
        .collect()
}
