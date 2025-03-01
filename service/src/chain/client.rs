use crate::config::service::ChainConfig;
use crate::config::ConfigService;
use crate::services::ServiceProvider;
use error_stack::{Report, Result, ResultExt};
use ethers::abi::Detokenize;
use ethers::contract::FunctionCall;
use ethers::prelude::transaction::eip2718::TypedTransaction::{Eip1559, Legacy};
use ethers::prelude::SignerMiddleware;
use ethers::providers::{Http, Middleware, Provider, RetryClient, RetryClientBuilder};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::{Address, TransactionReceipt, U256};
use lib::error::Error;
use reqwest::Url;
use std::borrow::Borrow;
use std::ops::Mul;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, warn};

pub type ChainClient = SignerMiddleware<Provider<RetryClient<Http>>, LocalWallet>;

/// Initialize RPC client for provided chain configuration
pub fn init_client(config: &ChainConfig) -> Result<Arc<ChainClient>, Error> {
    let rpc_url = Url::parse(&config.rpc).change_context(Error::Unknown)?;

    let http_client = Http::new(rpc_url);

    // TODO: Move retry policy to config
    let retry_client = RetryClientBuilder::default()
        .rate_limit_retries(10)
        .timeout_retries(3)
        .initial_backoff(Duration::from_millis(500))
        .compute_units_per_second(660)
        .build(
            http_client,
            Box::<ethers::providers::HttpRateLimitRetryPolicy>::default(),
        );

    let retry_provider = Provider::<RetryClient<Http>>::new(retry_client);

    let wallet = config
        .keeper
        .private_key
        .parse::<LocalWallet>()
        .change_context(Error::Unknown)?
        .with_chain_id(config.chain_id);

    let provider = SignerMiddleware::new(retry_provider, wallet);

    Ok(Arc::new(provider))
}

/// Call contract method or report error to discord
pub async fn call_contract_or_report<B, M, D>(
    services: ServiceProvider,
    client: Arc<ChainClient>,
    config: ChainConfig,
    address: Address,
    method: FunctionCall<B, M, D>,
    value: Option<U256>,
) -> Result<TransactionReceipt, Error>
where
    B: Borrow<M>,
    M: Middleware + 'static,
    D: Detokenize,
{
    // Estimate gas for provided method call, or fallback to 1_500_000
    let estimated_gas: U256 = match method.estimate_gas().await {
        Ok(estimated_gas) => estimated_gas.mul(2),
        Err(e) => {
            error!("Failed to estimate gas: {e:?}");
            match config.name.as_str() {
                "Ethereum" => U256::from(1_500_000_u64),
                _ => U256::from(5_000_000_u64),
            }
        }
    };

    let mut method = method.gas(estimated_gas);

    let gas_price = client.get_gas_price().await.map_err(|e| {
        Report::new(Error::Unknown).attach_printable(format!("Failed to fetch gas price: {e:?}"))
    })?;

    if gas_price.is_zero() {
        return Err(Report::new(Error::Unknown).attach_printable("Gas price is zero"));
    }

    let gas_price: U256 = gas_price.mul(12) / 10;
    method.tx = match method.tx {
        Legacy(tx) => Legacy(tx.gas_price(gas_price)),
        Eip1559(tx) => {
            let tx = tx.max_fee_per_gas(gas_price);
            Eip1559(tx.max_priority_fee_per_gas(U256::one()))
        }
        _ => {
            warn!("Unsupported transaction type");
            return Err(
                Report::new(Error::Unknown).attach_printable("Unsupported transaction type")
            );
        }
    };

    let result = async {
        // Simulate transaction
        method.call().await.change_context(Error::Unknown)?;

        method
            .send()
            .await
            .change_context(Error::Unknown)?
            .confirmations(1)
            .await
            .change_context(Error::Unknown)?
            .ok_or(Report::new(Error::Unknown))
    };

    match result.await {
        Ok(result) => Ok(result),
        Err(e) => {
            warn!("Failed to call contract: {e:?}");

            Err(e)
        }
    }
}
