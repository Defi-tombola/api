//! # Stream Provider
//!
//! The stream provider allow to consume events from chain and process them.
use ethers::prelude::EthEvent;
use chrono::{DateTime, Utc};
use error_stack::{Result, ResultExt};
use ethers::types::{Address, BlockNumber, Filter, U256, U64};
use lib::error::Error;
use service::chain::provider::ChainProvider;
use std::fmt::Debug;
use std::future::IntoFuture;
use std::{borrow::BorrowMut, sync::atomic};

use futures::{Stream, StreamExt};
use pin_project::pin_project;
use serenity::async_trait;
use std::pin::Pin;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn, Instrument};

use crate::events::*;
use service::common::{
    atomic::{await_signal, SignalFlag},
    shutdown::{await_shutdown_signal, ShutdownFlag},
};

/// Represents a flag used to signal the termination of the stream.
type TerminateFlag = SignalFlag;

/// Represents outcome of initialization of stream provider
pub type StreamProviderResult = tokio::task::JoinHandle<Result<(), Error>>;

/// The future for chain stream which read data from chain and send to internal channel
pub type ChainStreamFuture = Pin<Box<tokio::task::JoinHandle<Result<(), Error>>>>;

/// Defines an interface for processing on-chain events
pub trait StreamProvider
where
    Self: ChainProvider,
{
    fn start(self, shutdown: ShutdownFlag) -> StreamProviderResult;
}

/// Facilitates communication between a chain stream and it's provider.
pub struct Channel<T> {
    pub tx: mpsc::UnboundedSender<T>,
    pub rx: mpsc::UnboundedReceiver<T>,
}

impl<T> Default for Channel<T> {
    fn default() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self { tx, rx }
    }
}

/// Represents events communicated through the chain stream channel.
///
/// - `Event(T)`: Wraps an event of generic type `T` received from the chain.
/// - `Stop`: Serves as a signal to indicate that the stream should cease
///    operation, facilitating graceful shutdown or pause functionality.
pub enum ChannelEvent<T> {
    /// An event of type `T` received from the chain.
    Event(T),

    /// A signal to stop chain stream.
    Stop,
}

/// The `ChainStream` is used to read events from chain, validate and transform them.
/// Valid events are sent to the channel, where on another side are consumed over `Stream`
///
/// # Example:
///
/// ```rust
/// let mut stream = ChainStream::<ChainSubscription, EventTransformer, EventValidator>::init(
///         ChainSubscription,
///         EventTransformer,
///         EventValidator,
///         shutdown,
///     );
///
/// stream.start(
///     104075777,
///     vec!["0xc9155e8102e2c080ee8363f00762bdfeedc9e7f1".to_string()],
/// );
///
/// while let Some(event) = stream.next().await {
///    info!(?event, "Received event from chain");
/// }
/// ```

#[pin_project]
pub struct ChainStream<S, T, V>
where
    S: Subscription + Clone + Send,
    T: Transformer<S>,
    V: Validator<S>,
{
    stream: S,
    transformer: T,
    validator: V,
    pub shutdown: ShutdownFlag,
    pub terminate: TerminateFlag,
    pub channel: Channel<ChannelEvent<S::Item>>,
    pub future: Option<ChainStreamFuture>,
}

impl<S, T, V> ChainStream<S, T, V>
where
    S: Subscription + Send + Clone + 'static,
    T: Transformer<S>,
    V: Validator<S> + 'static,
{
    pub fn init(stream: S, transformer: T, validator: V, shutdown: ShutdownFlag) -> Self {
        Self {
            stream,
            transformer,
            validator,
            shutdown,
            terminate: TerminateFlag::default(),
            channel: Channel::default(),
            future: None,
        }
    }

    /// Start the chain stream
    ///
    /// This will start the stream, read logs from chan and send them to the channel.
    /// To gracefully stop the stream, you can use `stop` method.
    pub fn start(&mut self, from_block: u64, address: Vec<String>) {
        // Read all pending messages from channel to clear it
        while self.channel.rx.try_recv().is_ok() {}

        self.terminate
            .store(false, std::sync::atomic::Ordering::Release);

        let filter = SubscriptionFilter {
            from_block: BlockNumber::Number(from_block.into()),
            address,
        };

        info!(filter = ?filter, "Starting chain stream");

        self.stream = self.stream.with_filter(filter);
        self.future = Some(Box::pin(tokio::spawn({
            consume::<S, V>(
                self.stream.clone(),
                self.validator.clone(),
                self.channel.tx.clone(),
                self.shutdown.clone(),
                self.terminate.clone(),
            )
            .in_current_span()
        })));
    }

    /// Stop the chain stream
    ///
    /// This will stop the stream and wait for it to finish. You can use this method to gracefully
    /// stop the stream to re-start it later or to stop it completely.
    pub async fn stop(&mut self) {
        if !self.shutdown.load(std::sync::atomic::Ordering::Acquire) {
            self.terminate
                .store(true, std::sync::atomic::Ordering::Release);
        }

        if let Some(future) = self.future.borrow_mut() {
            let _ = future.into_future().await;
        }
    }
}

/// Implement `Stream` trait for `ChainStream`
///
/// Check `ChainStream` definition for more details how to use it.
impl<S, T, V> Stream for ChainStream<S, T, V>
where
    S: Subscription + Clone + Send,
    T: Transformer<S>,
    V: Validator<S>,
{
    type Item = ChainEvent;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        loop {
            // Stop reading events from chain if shutdown signal is received
            if self.shutdown.load(atomic::Ordering::Acquire) {
                info!("Shutdown signal received, stopping chain stream");
                return std::task::Poll::Ready(None);
            }

            // Pause reading events from chain if terminate signal is received
            if self.terminate.load(atomic::Ordering::Acquire) {
                info!("Terminate signal received, pausing chain stream");
                return std::task::Poll::Pending;
            }

            let input = match self.as_mut().project().channel.rx.poll_recv(cx) {
                std::task::Poll::Ready(Some(item)) => item,
                std::task::Poll::Ready(None) => return std::task::Poll::Ready(None),
                std::task::Poll::Pending => return std::task::Poll::Pending,
            };

            match input {
                ChannelEvent::Event(event) => match T::transform(event.clone()) {
                    Ok(event) => return std::task::Poll::Ready(Some(event)),
                    Err(e) => {
                        warn!(reason = %e, payload = ?event, "Failed to transform event");
                        continue;
                    }
                },
                // The `UnboundedReceiver` pauses in a `Poll::Pending` state, awaiting new messages.
                // To effectively manage shutdown or termination signals during this wait, a final
                // message is necessary. This allows the receiver to exit the `Poll::Pending` state
                // and check for `ShutdownFlag` or `TerminateFlag` conditions, ensuring graceful
                // handling of stream completion or termination requests.
                ChannelEvent::Stop => {
                    continue;
                }
            }
        }
    }
}

/// Define subscriber interface for chain data reader.
#[async_trait]
pub trait Subscription {
    type Item: Debug + Send + Clone;
    type Error: Debug + Send;

    /// Construct a new [`Subscription`] with provided filters
    fn with_filter(&self, filter: SubscriptionFilter) -> Self;

    /// Retrieves follow-up events related to a specified event within the same block.
    ///
    /// This method takes an event and, if it is designated as a follow-up event type, uses
    /// the address specified within that event to fetch all related events occurring in the
    /// same block.
    // async fn get_follow_up_events(&self, event: &Self::Item) -> Result<Vec<Self::Item>, Error>;

    /// Get stream of events
    async fn get_stream<'a>(
        &'a self,
    ) -> Pin<Box<dyn Stream<Item = std::result::Result<Self::Item, Self::Error>> + Send + 'a>>;
}

/// Specifies filters for subscribing to on-chain data.
///
/// Use this to customize the scope of data subscriptions, such as focusing on events from certain
/// addresses or starting from a specific block number.
///
/// # Fields
/// - `from_block`: The starting block number from which to begin receiving events.
/// - `address`: A list of addresses of interest. Only events involving these addresses
///   will be included in the subscription.
#[derive(Debug, Clone)]
pub struct SubscriptionFilter {
    pub from_block: BlockNumber,
    pub address: Vec<String>,
}

impl From<SubscriptionFilter> for Filter {
    fn from(filter: SubscriptionFilter) -> Self {
        Filter::new()
            .from_block(filter.from_block)
            .address(
                filter
                    .address
                    .iter()
                    .map(|address| address.parse().expect("Invalid address"))
                    .collect::<Vec<Address>>(),
            )
    }
}

/// Represent generic event received from chain and transformed to common format.
#[derive(Debug, Clone)]
pub struct ChainEvent {
    pub block_number: U64,
    pub transaction_hash: ethers::types::H256,
    pub log_index: U256,
    pub src_address: Address,
    pub dst_address: Address,
    pub kind: ChainEventKind,
    pub triggered_at: DateTime<Utc>,
}

// TODO: Provide more details about each event
#[derive(Debug, Clone)]
pub enum ChainEventKind {
    LotteryOpened(LotteryOpened),
    LotteryClosed(LotteryClosed),
    TicketBought(TicketBought),
    WinnerPaid(WinnerPaid),
    LotteryNumberGenerated(LotteryNumberGenerated),
    LotteryCanceled(LotteryCanceled),
    FeeCollected(FeeCollected)
}

/// Provides a mechanism for transforming events from a specific subscription into a standardized format.
pub trait Transformer<Sub: Subscription> {
    /// Transforms a subscription-specific event into a common `ChainEvent`.
    fn transform(input: Sub::Item) -> Result<ChainEvent, Error>;
}

/// Provides a mechanism for validating events from a specified subscription
#[async_trait]
pub trait Validator<Sub: Subscription>
where
    Self: Clone + Send,
{
    /// Validate a subscription-specific event.
    async fn validate(&self, input: &Sub::Item) -> Result<(), Error>;
}

/// Processes a provided chain stream, applying a validator and transformer to each item, and
/// forwards valid messages to a specified channel.
///
/// This operation continues until either `shutdown` or `terminate` flags are set to `TRUE`, signaling
/// the task to cease.
///
/// NOTE: If the channel is dropped and we are unable to send a `Send` message, the application will be
/// forced to stop via `std::process::exit(1)`.
async fn consume<S, V>(
    stream: S,
    validator: V,
    channel: mpsc::UnboundedSender<ChannelEvent<S::Item>>,
    shutdown: ShutdownFlag,
    terminate: TerminateFlag,
) -> Result<(), Error>
where
    S: Subscription + Send,
    V: Validator<S> + 'static,
{
    let mut inner_stream = stream.get_stream().await;

    loop {
        tokio::select! {
            item = inner_stream.next() => {
                match item {
                    Some(Ok(event)) => {
                        // Verify that messages is valid to be further processed, that includes filtering
                        // of duplicates, and invalid messages
                        if let Err(e) = validator.validate(&event).await { // EventEmitted
                            warn!(reason = ?e, "Failed to validate event");
                            continue;
                        }

                        if let Err(e) = channel.send(ChannelEvent::Event(event.clone())) {
                            error!("Failed to send follow up event to channel. Event: {:?}. Error: {:?}",
                                event.clone(),
                                e.to_string()
                            );
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        warn!(reason = ?e, "Received error from chain subscription, stopping chain consumer");
                        break;
                    }
                    None => {
                        debug!("Chain subscription has ended, stopping chain consumer");
                    }
                }
            }
            _ = await_signal(terminate.clone(), 2) => {
                info!("Terminate signal received, stopping chain consumer");
                break;
            }
            _ = await_shutdown_signal(shutdown.clone()) => {
                info!("Shutdown signal received, stopping chain consumer");

                if let Err(e) = channel.send(ChannelEvent::Stop) {
                    error!(reason = ?e, "Failed to send terminate signal to channel, force to stop system");
                    error!("Full error: {:?}", e);
                    std::process::exit(1);
                } else {
                    break;
                }
            }
        }
    }

    info!("Chain consumer stopped");

    Ok(())
}
