use error_stack::Result;
use lib::error::Error;
use std::sync::Arc;

use crate::config::service::ChainConfig;

use super::ChainClient;

/// Common trait for all chain providers
pub trait ChainProvider
where
    Self: Send + Sync,
{
    /// Configures a name for [`ChainProvider`] which is used to identify the chain and should be unique
    fn name(&self) -> String;

    /// Returns a client for the chain using current configuration
    fn get_client(&self) -> Result<Arc<ChainClient>, Error>;

    /// Returns the chain configuration
    fn get_config(&self) -> Arc<ChainConfig>;

    /// Returns a client for the chain using provided configuration
    fn get_client_with_config(config: &ChainConfig) -> Result<Arc<ChainClient>, Error>;
}
