use crate::config::service::{ConfigService, RedisConfig};
use crate::prelude::ServiceProvider;
use crate::services::ServiceFactory;
use entity::prelude::{TicketModel, PrizeModel};
use error_stack::{Result, ResultExt};
use futures::stream::StreamExt;
use lib::error::Error;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use redis::Client;
use serde::Serialize;
use serenity::async_trait;
use std::sync::Arc;
use tokio::sync::broadcast::Receiver;
use tokio_stream::wrappers::BroadcastStream;
use tracing::{error, info, warn};

struct MessageBrokerServiceInner {
    connection: ConnectionManager,
    channel: Receiver<Event>,
}

// TODO: Need to rename to something like MessageBus or MessageBroker, or even maybe Publisher

#[derive(Clone)]
pub struct MessageBrokerService(Arc<MessageBrokerServiceInner>);

#[derive(Clone, Debug)]
pub enum Event {
    TicketBought(TicketModel),
    PrizePoolUpdated(PrizeModel),
}

impl MessageBrokerService {
    pub async fn new(config: RedisConfig) -> Result<Self, Error> {
        info!(url = config.url, "Connecting to redis");

        let client = Client::open(config.url.clone()).change_context(Error::Redis)?;

        let connection = client
            .get_tokio_connection_manager()
            .await
            .change_context(Error::RedisConnect)?;

        let (tx, rx) = tokio::sync::broadcast::channel(10_000);

        let conn = client.get_tokio_connection().await.unwrap();
        let mut pubsub = conn.into_pubsub();
        pubsub.subscribe("ticket_bought").await.unwrap();

        tokio::spawn(async move {
            let mut stream = pubsub.into_on_message();

            while let Some(msg) = stream.next().await {
                let payload = &msg.get_payload::<String>();
                if let Err(e) = payload {
                    warn!("Failed to get payload: {e:?}");
                    continue;
                }

                let payload = payload.as_ref().unwrap();

                let event = match msg.get_channel_name() {
                    "ticket_bought" => Some(Event::TicketBought(
                        serde_json::from_str::<TicketModel>(payload).unwrap(),
                    )),
                    _ => None,
                };

                if let Some(event) = event {
                    if let Err(e) = tx.send(event.clone()) {
                        error!("Failed to broadcast event: {}", e);
                        error!("Event: {:?}", event);
                    }
                } else {
                    warn!(
                        "Received message from not supported channel {}",
                        msg.get_channel_name()
                    );
                }
            }
        });

        Ok(MessageBrokerService(Arc::new(MessageBrokerServiceInner {
            connection,
            channel: rx,
        })))
    }

    pub async fn send<T>(&self, channel: String, msg: T) -> Result<(), Error>
    where
        T: Serialize,
    {
        let mut conn = self.0.connection.clone();
        conn.publish(channel, serde_json::to_string(&msg).unwrap())
            .await
            .change_context(Error::Redis)?;

        Ok(())
    }

    pub async fn subscribe(&self) -> impl futures::Stream<Item = Event> {
        let stream = BroadcastStream::from(self.0.channel.resubscribe());

        stream.filter_map(|event| {
            futures::future::ready(match event {
                Ok(event) => Some(event),
                _ => None,
            })
        })
    }
}

#[async_trait]
impl ServiceFactory for MessageBrokerService {
    async fn factory(services: ServiceProvider) -> Result<Self, Error> {
        let config = services.get_service_unchecked::<ConfigService>().await;
        Self::new(config.redis.clone()).await
    }
}
