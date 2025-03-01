use async_graphql::{Context, Object, Subscription};
use futures::{Stream, StreamExt};
use service::{message_broker::{Event, MessageBrokerService}, prelude::{ServiceProvider, StoreService}, ticket::store::TicketStore};
use tracing::{info, warn};

use super::types::TicketType;

#[derive(Default)]
pub struct TicketQuery;

#[Object]
impl TicketQuery {
    /// Fetches the last 10 tickets sold
    async fn get_last_tickets(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<TicketType>> {
        let services = ctx.data_unchecked::<ServiceProvider>();
        let store_service = services.get_service_unchecked::<StoreService>().await;

        let pool = store_service.read();
        let tickets = TicketStore::find_last_bought_tickets(pool, 10 as i32).await.map_err(|e| {
            warn!("Failed to get last tickets: {e:?}");
            async_graphql::Error::from("Internal error")
        })?;
        
        Ok(tickets.into_iter().map(Into::into).collect())
    }
    
    async fn get_user_tickets(&self, ctx: &Context<'_>, address: String) -> async_graphql::Result<Vec<TicketType>> {
        let services = ctx.data_unchecked::<ServiceProvider>();
        let store_service = services.get_service_unchecked::<StoreService>().await;

        let pool = store_service.read();
        let tickets = TicketStore::find_by_address(pool, address).await.map_err(|e| {
            warn!("Failed to get user tickets: {e:?}");
            async_graphql::Error::from("Internal error")
        })?;
        
        Ok(tickets.into_iter().map(Into::into).collect())
    }
}

#[derive(Default)]
pub struct TicketSubscription;

#[Subscription]
impl TicketSubscription {
    async fn ticket_bought(&self, ctx: &Context<'_>) -> async_graphql::Result<impl Stream<Item = TicketType>> {
        let services = ctx.data_unchecked::<ServiceProvider>();
        let broker = services.get_service_unchecked::<MessageBrokerService>().await;

        Ok(broker.subscribe().await.filter_map(|event| {
            futures::future::ready(match event {
                Event::TicketBought(ticket) => {
                    info!("Ticket bought: {ticket:?}");
                    Some(ticket.into())
                },
                _ => None,
            })
        }))
    }
}
