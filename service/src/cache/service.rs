use crate::config::service::{ConfigService, RedisConfig};
use crate::prelude::ServiceProvider;
use crate::services::ServiceFactory;
use error_stack::{Result, ResultExt};
use lib::error::Error;
use redis::aio::Connection;
use redis::Client;
use serenity::async_trait;
use std::ops::Deref;
use tracing::info;

#[derive(Clone)]
pub struct CacheService(Client);

impl CacheService {
    pub fn new(config: RedisConfig) -> Result<Self, Error> {
        info!(url = config.url, "Connecting to redis");

        let client = Client::open(config.url).change_context(Error::Redis)?;

        Ok(CacheService(client))
    }

    pub async fn get_connection(&self) -> Result<Connection, Error> {
        self.0
            .get_tokio_connection()
            .await
            .change_context(Error::RedisConnect)
    }
}

impl Deref for CacheService {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl ServiceFactory for CacheService {
    async fn factory(services: ServiceProvider) -> Result<Self, Error> {
        let config = services.get_service_unchecked::<ConfigService>().await;
        Self::new(config.redis.clone())
    }
}
