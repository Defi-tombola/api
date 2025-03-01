use error_stack::{Report, Result, ResultExt};
use ethers::types::Address;
use serde::{Deserialize, Serialize};
use serenity::async_trait;
use std::{
    collections::HashMap, fmt::Display, fs, ops::Deref, path::Path, str::FromStr, sync::Arc,
};

use crate::prelude::ServiceProvider;
use crate::services::ServiceFactory;
use lib::error::Error;

#[derive(Debug, Serialize)]
pub struct ConfigServiceInner {
    pub environment: AppEnvironment,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub graphql: GQLConfig,
    pub chains: Vec<ChainConfig>,
    pub jwt: JWTConfig,
    pub twitter: TwitterConfig,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigService(Arc<ConfigServiceInner>);

impl<'de> serde::Deserialize<'de> for ConfigService {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize, Default)]
        struct AdHocConfig {
            pub environment: AppEnvironment,
            pub database: DatabaseConfig,
            pub redis: RedisConfig,
            pub graphql: GQLConfig,
            pub chains: Vec<ChainConfig>,
            pub jwt: JWTConfig,
            pub twitter: TwitterConfig,
        }

        let ad_hoc: AdHocConfig = serde::Deserialize::deserialize(deserializer)?;

        ConfigService::builder()
            .environment(ad_hoc.environment)
            .database(ad_hoc.database)
            .redis(ad_hoc.redis)
            .graphql(ad_hoc.graphql)
            .chains(ad_hoc.chains)
            .jwt(ad_hoc.jwt)
            .twitter(ad_hoc.twitter)
            .build()
            .map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

impl FromStr for ConfigService {
    type Err = Report<Error>;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_yaml::from_str(s).change_context(Error::ConfigInvalid)
    }
}

impl Display for ConfigService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_yaml::to_string(&self).unwrap())
    }
}

impl Deref for ConfigService {
    type Target = ConfigServiceInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ConfigService {
    pub fn inner(&self) -> Arc<ConfigServiceInner> {
        self.0.clone()
    }

    pub fn try_get_chain_config_by_chain_id(&self, chain_id: u32) -> Option<&ChainConfig> {
        self.0.chains.iter().find(|c| c.chain_id == chain_id)
    }
    
    pub fn try_get_chain_config_by_chain_name(&self, chain_name: &str) -> Option<&ChainConfig> {
        self.0.chains.iter().find(|c| c.name == chain_name)
    }
}

#[buildstructor::buildstructor]
impl ConfigService {
    #[builder]
    pub fn new(
        environment: Option<AppEnvironment>,
        database: Option<DatabaseConfig>,
        redis: Option<RedisConfig>,
        graphql: Option<GQLConfig>,
        chains: Option<Vec<ChainConfig>>,
        jwt: Option<JWTConfig>,
        twitter: Option<TwitterConfig>,
    ) -> Result<Self, Error> {
        let inner = ConfigServiceInner {
            environment: environment.unwrap_or_default(),
            database: database.unwrap_or_default(),
            redis: redis.unwrap_or_default(),
            graphql: graphql.unwrap_or_default(),
            chains: chains.unwrap_or_default(),
            jwt: jwt.unwrap_or_default(),
            twitter: twitter.unwrap_or_default(),
        };

        Ok(ConfigService(Arc::new(inner)))
    }

    pub fn read_file(path: &Path) -> Result<Self, Error> {
        let config = fs::read_to_string(path)
            .change_context(Error::ConfigNotFound(path.to_str().unwrap().to_string()))?;

        config.parse()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct AppEnvironment {
    pub name: String,
    pub otlp_grcp_endpoint: String,
    pub otlp_http_endpoint: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct AppConfig {
    pub base_url: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct ChainConfig {
    pub name: String,
    pub chain_id: u32,
    pub rpc: String,
    pub block_number: i32,
    pub explorer_url: String,
    pub contracts: HashMap<String, Address>,
    pub keeper: KeeperConfig,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct GQLConfig {
    pub listen: String,
    pub endpoint: String,
    pub subscription_endpoint: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct DatabaseConfig {
    pub servers: Vec<DatabaseConfigServer>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct DatabaseConfigServer {
    pub url: String,
    pub read_only: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct DiscordConfig {
    pub internal: DiscordInternalConfig,
    pub external: DiscordExternalConfig,
    pub webhook_url: String,
    pub avatar_url: String,
    pub enabled: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct DiscordInternalConfig {
    pub token: String,
    pub enabled: bool,
    pub channels: HashMap<String, String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct DiscordExternalConfig {
    pub token: String,
    pub enabled: bool,
    pub channels: HashMap<String, String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct TelegramConfig {
    pub enabled: bool,
    pub bot_token: String,
    pub api_id: i32,
    pub api_hash: String,
    pub db_encryption_key: String,
    pub db_remote_path: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct KeeperConfig {
    pub private_key: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct TasksConfig {
    pub ohlc_archive: TaskConfig,
    pub vault_balance: TaskConfig,
    pub vault_shares: TaskConfig,
    pub vault_bridge: TaskConfig,
    pub asset_prices: TaskConfig,
    pub asset_supply: TaskConfig,
    pub vault_stats: TaskConfig,
    pub price_checker: TaskConfig,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct TaskConfig {
    pub interval: u64,
    pub health_timeout: u64,
    pub max_retries: u8,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct AWSConfig {
    pub region: String,
    pub s3: AWSS3Config,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct AWSS3Config {
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
    pub bucket_url: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct JWTConfig {
    pub private_key: String,
    pub public_key: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct TenderlyConfig {
    pub access_key: String,
    pub project_slug: String,
    pub user: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct TwitterConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct CryptoCompareConfig {
    pub api_key: String,
    pub historical_data_days: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct CoinGeckoConfig {
    pub api_key: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct ImpactConfig {
    pub account_sid: String,
    pub auth_token: String,
    pub campaign_id: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct ZeroXConfig {
    pub api_key: String,
}

#[async_trait]
impl ServiceFactory for ConfigService {
    async fn factory(_services: ServiceProvider) -> Result<Self, Error> {
        Err(Report::new(Error::Unknown)
            .attach_printable("ConfigService should be initialized manually"))
    }
}
