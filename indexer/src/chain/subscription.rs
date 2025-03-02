use async_trait::async_trait;
use error_stack::{Result, ResultExt};
use ethers::prelude::{EthEvent, Middleware};
use ethers::providers::{LogQueryError, ProviderError};
use ethers::types::{BlockNumber, Filter, Log, U64};
use futures::{stream, Stream, StreamExt};
use lib::error::Error;
use service::chain::ChainClient;
use tokio::time::interval;
use tracing::info;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use crate::stream::{Subscription, SubscriptionFilter};

/// Implement [`Subscription`]
#[derive(Clone)]
pub struct ChainSubscription {
    pub client: Arc<ChainClient>,
    pub filter: Option<SubscriptionFilter>,
}

#[async_trait]
impl Subscription for ChainSubscription {
    type Item = Log;
    type Error = LogQueryError<ProviderError>;

    fn with_filter(&self, filter: SubscriptionFilter) -> Self {
        Self {
            client: self.client.clone(),
            filter: Some(filter),
        }
    }

    // async fn get_follow_up_events(&self, event: &Self::Item) -> Result<Vec<Self::Item>, Error> {
    //     if let Some(signature) = event.topics.first() {
    //         if EventEmitted::signature().eq(signature) {
    //             let payload = EventEmitted::decode_log(&event.clone().into())
    //                 .change_context(Error::EventDecodeFailed)?;

    //             let address = payload.address;
    //             let block_number = event.block_number.expect("Block number is missing");

    //             let filter = Filter::new()
    //                 .from_block(BlockNumber::Number(block_number))
    //                 .to_block(BlockNumber::Number(block_number))
    //                 .address(vec![address]);

    //             return self
    //                 .client
    //                 .get_logs(&filter)
    //                 .await
    //                 .change_context(Error::Unknown);
    //         }
    //     }

    //     return Ok(vec![]);
    // }

    async fn get_stream<'a>(
        &'a self,
    ) -> Pin<Box<dyn Stream<Item = std::result::Result<Self::Item, Self::Error>> + Send + 'a>> {
        let filter = self
            .filter
            .as_ref()
            .map(|i| Filter::from(i.clone()))
            .unwrap_or_default();
        
        let client = self.client.clone();
        let poll_interval = Duration::from_secs(5);
        
        let start_block = {
            match filter.get_from_block() {
                Some(block) => BlockNumber::Number(block),
                None => BlockNumber::Latest,
            }
        };
        info!("Starting logs stream from block {:?}", start_block);
        
        // Create a stream that polls for logs at regular intervals
        let stream_logs = stream::unfold((client, filter, start_block), move |(client, filter, mut from_block)| async move {
            // Sleep for the poll interval
            info!("Sleeping for {:?}", poll_interval);
            interval(poll_interval).tick().await;
            info!("Polling for logs");
            
            let latest_block = client
                .get_block_number()
                .await
                .change_context(Error::Unknown)
                .ok()?;
            
            let mut updated_filter = filter.clone();
            if from_block.is_latest() {
                updated_filter = updated_filter.from_block(BlockNumber::Number(latest_block - U64::from(10)));
            } else {
                updated_filter = updated_filter.from_block(from_block);
            }
            
            let to_block = match from_block.as_number() {
                Some(block_num) => {
                    let upper_bound = block_num + U64::from(100);
                    if upper_bound > latest_block {
                        BlockNumber::Latest
                    } else {
                        BlockNumber::Number(upper_bound)
                    }
                }
                None => BlockNumber::Latest,
            };
            
            updated_filter = updated_filter.to_block(to_block);
            
            info!("Fetching logs using the updated filter");
            // Fetch logs using the updated filter
            let logs = client
                .get_logs(&updated_filter)
                .await
                .change_context(Error::Unknown)
                .ok()?;
            
            // Update the last block number
            from_block = to_block;
            
            info!("Emitting logs as a stream");
            // Emit the logs as a stream
            Some((stream::iter(logs.into_iter().map(Ok)), (client, filter, from_block)))
        }).flatten();
        
        info!("Logs stream created");
        
        Box::pin(stream_logs)
    }
}
