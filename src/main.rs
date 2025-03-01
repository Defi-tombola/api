mod cli;

use clap::Parser;
use cli::Cli;
use service::config::service::ConfigService;
use service::prelude::{ServiceProvider, StoreService};
use service::telemetry;
use sqlx::migrate::Migrator;
use std::path::Path;
use tracing::{error, info};

#[tokio::main]
#[tracing::instrument]
async fn main() {
    let args = Cli::parse();

    let config =
        ConfigService::read_file(Path::new(&args.config)).expect("Failed to read config file");

    let service_name = args.command.to_string();
    telemetry::init(
        &config,
        service_name,
        args.log_level.into(),
    )
    .expect("Failed to initialize telemetry");

    // Run pending migrations
    run_migrations(&config).await;

    let meter = telemetry::get_meter_provider().meter("server_start_up_times");

    let counter = meter.u64_counter("start_up_time").build();
    match args.command {
        cli::Commands::Indexer { chains, with_tasks } => {
            counter.clone().add(1, &[]);
            if let Err(e) = indexer::start(config, chains, with_tasks).await {
                error!(reason = ?e, "Failed to start indexer");
            }
        }
        cli::Commands::GraphQL => {
            counter.clone().add(1, &[]);
            if let Err(e) = graphql::start(config).await {
                error!(reason = ?e, "Failed to start GraphQL");
            }
        }
    }

    telemetry::shutdown().await.expect("Failed to shutdown telemetry");
}

async fn run_migrations(config: &ConfigService) {
    info!("Running migrations!");
    let services_provider = ServiceProvider::new();
    services_provider.add_service(config.to_owned()).await;
    let store_service = services_provider
        .get_service_unchecked::<StoreService>()
        .await;

    let pool = store_service.read();
    let migrations_path = Path::new("./migrations");
    let mut migrator = Migrator::new(migrations_path)
        .await
        .expect("Failed to load migrations");

    let migrator = migrator.set_locking(true);

    migrator.run(pool).await.expect("Failed to run migrations");
    
    info!("Migrations finished!");
}

