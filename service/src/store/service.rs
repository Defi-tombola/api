use error_stack::{Result, ResultExt};
use lib::error::Error;
use rand::seq::SliceRandom;
use serenity::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use tracing::info;

use crate::{
    config::service::{ConfigService, DatabaseConfig},
    prelude::ServiceProvider,
    services::ServiceFactory,
};

#[derive(Clone)]
pub struct StoreService {
    read_write: Vec<PgPool>,
    read_only: Vec<PgPool>,
}

pub type DatabaseTransaction<'c> = Transaction<'c, Postgres>;

impl StoreService {
    pub async fn new(config: DatabaseConfig) -> Result<Self, Error> {
        let mut read_only: Vec<PgPool> = Vec::new();
        let mut read_write: Vec<PgPool> = Vec::new();

        for server in config.servers {
            info!(
                url = server.url,
                read_only = server.read_only,
                "Connecting to database"
            );

            let db = PgPool::connect(&server.url)
                .await
                .change_context(Error::Store)?;

            if !server.read_only {
                read_write.push(db.clone());
            }

            read_only.push(db);
        }

        Ok(StoreService {
            read_only,
            read_write,
        })
    }

    pub fn read(&self) -> &PgPool {
        self.read_only.choose(&mut rand::thread_rng()).unwrap()
    }

    pub fn write(&self) -> &PgPool {
        if self.read_write.is_empty() {
            // TODO: Should we throw an error/panic here if no write instances are available?
            self.read()
        } else {
            self.read_write.choose(&mut rand::thread_rng()).unwrap()
        }
    }

    /// Create a new database transaction
    pub async fn begin_transaction(&self) -> Result<DatabaseTransaction, Error> {
        self.write().begin().await.change_context(Error::Store)
    }

    /// Commit provided transaction
    pub async fn commit_transaction(&self, db_tx: DatabaseTransaction<'_>) -> Result<(), Error> {
        db_tx.commit().await.change_context(Error::Store)
    }

    /// Rollback provided transaction
    pub async fn rollback_transaction(&self, db_tx: DatabaseTransaction<'_>) -> Result<(), Error> {
        db_tx.rollback().await.change_context(Error::Store)
    }
}

#[async_trait]
impl ServiceFactory for StoreService {
    async fn factory(services: ServiceProvider) -> Result<Self, Error> {
        let config = services.get_service_unchecked::<ConfigService>().await;

        Ok(StoreService::new(config.database.clone()).await.unwrap())
    }
}
