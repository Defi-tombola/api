mod subscription;
mod transformer;
mod validator;

use crate::chain::subscription::ChainSubscription;
use crate::chain::transformer::EventTransformer;
use crate::chain::validator::EventValidator;
use crate::handler::{Handler, HandlerPayload};
use crate::state::StateManager;
use crate::stream::{ChainEventKind, ChainStream, StreamProvider, StreamProviderResult};
use chrono::{TimeZone, Utc};
use futures::StreamExt;
use service::chain::provider::ChainProvider;
use service::chain::traits::string::ToHexString;
use service::chain::utils::get_block::get_timestamp_by_block;
use service::chain::Chain;
use service::prelude::StoreService;
use service::transaction::service::TransactionService;
use service::transaction::store::TransactionStore;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tracing::{debug, error, info, warn, Instrument};

/// Implement [`StreamProvider`] for common [`Chain`]
impl StreamProvider for Chain {
    fn start(self, shutdown: Arc<AtomicBool>) -> StreamProviderResult {
        let span = tracing::info_span!("chain", chain = self.config.name.as_str());

        tokio::spawn({
            async move {
                info!(rpc = self.config.rpc, "Spawning chain processor");

                let client = self.get_client()?;

                let mut stream =
                    ChainStream::<ChainSubscription, EventTransformer, EventValidator>::init(
                        ChainSubscription {
                            client: client.clone(),
                            filter: None,
                        },
                        EventTransformer,
                        EventValidator::new(self.services.clone()),
                        shutdown.clone(),
                    );

                let state_manager = StateManager::new(&self.config, self.services.clone()).await?;

                let state = state_manager.current().await;
                stream.start(state.block_number, state.address());

                // Todo rethink this flow, it works as is, but there should be a better way to do it
                // We cannot validate events on validator, since events validated there by DDBB, may result in
                // dupplicated events since we send the event to the channel, and validate the next event. This will result
                // in that the event is not fully processed and saved to database yet.
                while let Some(mut event) = stream.next().await {
                    let services = self.services.clone();
                    let store_service = services.get_service_unchecked::<StoreService>().await;
                    let pool = store_service.read();
                    
                    if let Some(transaction_log) =
                    TransactionStore::try_find_by_hash_and_log_index(
                        pool,
                        event.transaction_hash,
                        event.log_index,
                    )
                    .await?
                    {
                        let hash = transaction_log.transaction_hash;
                        let log_index = transaction_log.log_index;
                        info!("Event with hash {} already processed with log_index {}", hash, log_index);
                        continue;
                    }
                    
                    let event_context = HandlerPayload::from((event.clone(), event.kind.clone()))
                        .get_context(&self);
                    
                    let mut db_tx = store_service.begin_transaction().await?;
                    let timestamp =
                        get_timestamp_by_block(event.block_number, client.clone()).await?;
                    
                    event.triggered_at = Utc.timestamp_opt(timestamp as i64, 0).unwrap(); // Safe to unwrap
                    let transaction_service =
                        services.get_service_unchecked::<TransactionService>().await;
                    
                    let event_block_number = event.block_number.as_u64();
                    let event_handler_result = match event.kind.clone() {
                        ChainEventKind::LotteryOpened(kind) => {
                                self.handle(
                                    HandlerPayload::from((event.clone(), kind)),
                                    services,
                                    state_manager.clone(),
                                    &mut db_tx,
                                )
                                .await
                            }
                            ChainEventKind::LotteryClosed(kind) => {
                                self.handle(
                                    HandlerPayload::from((event.clone(), kind)),
                                    services,
                                    state_manager.clone(),
                                    &mut db_tx,
                                )
                                .await
                            }
                            ChainEventKind::TicketBought(kind) => {
                                self.handle(
                                    HandlerPayload::from((event.clone(), kind)),
                                    services,
                                    state_manager.clone(),
                                    &mut db_tx,
                                )
                                .await
                            }
                            ChainEventKind::WinnerPaid(kind) => {
                                self.handle(
                                    HandlerPayload::from((event.clone(), kind)),
                                    services,
                                    state_manager.clone(),
                                    &mut db_tx,
                                )
                                .await
                            }
                            ChainEventKind::LotteryNumberGenerated(kind) => {
                                self.handle(
                                    HandlerPayload::from((event.clone(), kind)),
                                    services,
                                    state_manager.clone(),
                                    &mut db_tx,
                                )
                                .await
                            }
                            ChainEventKind::LotteryCanceled(kind) => {
                                self.handle(
                                    HandlerPayload::from((event.clone(), kind)),
                                    services,
                                    state_manager.clone(),
                                    &mut db_tx,
                                )
                                .await
                            }
                            ChainEventKind::FeeCollected(kind) => {
                                self.handle(
                                    HandlerPayload::from((event.clone(), kind)),
                                    services,
                                    state_manager.clone(),
                                    &mut db_tx,
                                )
                                .await
                            }
                    };
                    
                    if let Err(e) = event_handler_result {
                        error!(
                            tx_hash = event.transaction_hash.to_hex_string(),
                            block_number = event.block_number.to_string(),
                            log_index = event.log_index.to_string(),
                            "Failed to handle chain event. Error: {:?}", e
                        );
                        break;
                    }
                    
                    let transaction = transaction_service
                        .create_without_side_effects(event_context, &mut db_tx)
                        .await?;
                    
                    debug!("Transaction log created: {:?}", transaction);
                    store_service.commit_transaction(db_tx).await?;
                    
                    state_manager.set_block_number(event_block_number).await?;
                    state_manager.save().await?;
                    
                    // Restart stream when state has changed
                    if let Some(state) = state_manager.next().await {
                        stream.stop().await;
                        stream.start(state.block_number, state.address());
                    }
                }

                // Graceful shutdown stream
                let _ = stream.stop().await;

                let last_block_number = state_manager.current().await.block_number;
                info!(last_block_number, "Chain processor stopped");

                Ok(())
            }
            .instrument(span)
        })
    }
}

// OLD CODE

// impl Chain {
// pub fn new(config: ChainConfig, services: Services) -> Self {
//     Self {
//         config: Arc::new(config),
//         services,
//     }
// }

// async fn start_stream(
//     &self,
//     client: Arc<ChainClient>,
//     shutdown: Arc<AtomicBool>,
// ) -> Result<(), Error> {
//     info!("Starting stream for chain {}", self.config.name);

//     let context = Arc::new(EventHandlerContext {
//         log_target: self.log_target.clone(),
//         config: self.config.clone(),
//         client: client.clone(),
//         services: self.services.clone(),
//     });

//     // Setup events handlers
//     let handlers = Handlers::new(context).await?;

//     // Setup stream and start worker
//     let last_block = self.get_last_block().await;
//     let stream = ChainStream::new(
//         self.log_target.clone(),
//         self.config.clone(),
//         client.clone(),
//         self.services.clone(),
//         last_block,
//     )
//     .await?;

//     stream.start_worker().await;

//     // Listen for updates from indexed vaults
//     let receiver = stream.get_receiver();
//     let mut reader = receiver.write().await;

//     let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));

//     loop {
//         tokio::select! {
//             _ = interval.tick() => {
//                 if shutdown.load(atomic::Ordering::Acquire) {
//                     break;
//                 }
//             }
//             details = reader.deref_mut().recv() => {
//                 if let Some(details) = details {
//                     let block_number = details.log.block_number.unwrap();

//                     match handlers.handle(details).await {
//                         Ok(feedback) => {
//                             if let Some(feedback) = feedback {
//                                 let result = match feedback {
//                                     HandlerFeedback::SubscribeAddress(address) => {
//                                         stream.subscribe_address(address).await
//                                     }
//                                     HandlerFeedback::UnsubscribeAddress(address) => {
//                                         stream.unsubscribe_address(address).await
//                                     }
//                                 };

//                                 if let Err(e) = result {
//                                     warn!(target: &self.log_target, "Failed to handle feedback: {:?}", e);
//                                 }
//                             }

//                             self.services
//                                 .get_service_unchecked::<StateService>()
//                                 .save_last_block(&self.config.name, block_number)
//                                 .await;

//                             debug!(target: &self.log_target, "Saved last processed block #{}", 0);
//                         }
//                         Err(e) => {
//                             error!(target: &self.log_target, "Failed to handle event: {:?}", e);
//                         }
//                     }
//                 }
//             }
//         }
//     }

//     info!(target: &self.log_target, "Stream of chain {} stopped", self.config.name);

//     Ok(())
// }

// /// Get last processed block from DB, or return one specified in config
// async fn get_last_block(&self) -> U64 {
//     let config_block_number: U64 = self.config.block_number.into();

//     let state = self.services.get_service_unchecked::<StateService>();
//     state
//         .get_last_block(&self.config.name)
//         .await
//         .map(|last_block| {
//             if last_block.gt(&config_block_number) {
//                 last_block
//             } else {
//                 config_block_number
//             }
//         })
//         .unwrap_or(config_block_number)
// }

// fn transport() ->
// }

// enum EventKind {
//     VaultChildCreated(VaultChildCreated),
// }

// pub struct ChainEvent {
//     name: String,
//     signature: String,
//     kind: EventKind,
// }

// /// Stream chain events related to the protocol
// impl Stream for Chain {
//     type Item = ChainEvent;

//     fn poll_next(
//         self: std::pin::Pin<&mut Self>,
//         _cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Option<Self::Item>> {
//         std::task::Poll::Ready(None)
//     }
// }
