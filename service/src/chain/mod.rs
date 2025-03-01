pub mod client;
pub mod provider;
pub mod stream;
pub mod traits;
pub mod types;
pub mod utils;
pub mod contract;

use std::sync::Arc;

pub use client::*;
pub use stream::*;

use crate::{config::service::ChainConfig, prelude::ServiceProvider};
use error_stack::Result;
use lib::error::Error;

use self::provider::ChainProvider;

pub struct Chain {
    pub config: Arc<ChainConfig>,
    pub services: ServiceProvider,
}

/// Helper for Chain initialization, which create Chain instance from ChainConfig and Services
///
/// # Example
///
/// ```rust
/// let chain = Chain::from((config, services));
/// ```
impl From<(ChainConfig, ServiceProvider)> for Chain {
    fn from((config, services): (ChainConfig, ServiceProvider)) -> Self {
        Self {
            config: Arc::new(config),
            services,
        }
    }
}

impl ChainProvider for Chain {
    fn name(&self) -> String {
        self.config.name.clone()
    }

    fn get_client(&self) -> Result<Arc<ChainClient>, Error> {
        Self::get_client_with_config(&self.config)
    }

    fn get_config(&self) -> Arc<ChainConfig> {
        self.config.clone()
    }

    fn get_client_with_config(config: &ChainConfig) -> Result<Arc<ChainClient>, Error> {
        init_client(config)
    }
}
