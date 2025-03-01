use error_stack::Result;
use lib::error::Error;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::fmt;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::info;

use uuid::Uuid;

use crate::store::service::StoreService;

/// A type that can be stored as an [`Event<E>`] resource.
/// You can conveniently access events using the [`EventReader`] and [`EventWriter`] system parameter.
///
/// Events must be thread-safe.
pub trait Event: Send + Sync + Debug + Serialize + DeserializeOwned + 'static {}

/// An `EventId` uniquely identifies an [`Event`] of type `E`.
pub struct EventId<E: Event> {
    pub id: Uuid,
    event: PhantomData<E>,
}

impl<E: Event> Copy for EventId<E> {}

impl<E: Event> Clone for EventId<E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E: Event> PartialEq for EventId<E> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<E: Event> Eq for EventId<E> {}

impl<E: Event> fmt::Display for EventId<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Self as fmt::Debug>::fmt(self, f)
    }
}

impl<E: Event> fmt::Debug for EventId<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "event<{}>#{}",
            std::any::type_name::<E>().split("::").last().unwrap(),
            self.id
        )
    }
}

#[allow(dead_code)]
struct EventInstance<E: Event> {
    pub id: EventId<E>,
    pub event: E,
}

#[allow(dead_code)]
pub struct EventStore {
    store: Arc<StoreService>,
    events: Vec<Value>,
}

impl EventStore {
    pub fn new(store: Arc<StoreService>) -> Self {
        Self {
            store,
            events: Default::default(),
        }
    }
}

#[allow(dead_code)]
impl EventStore {
    async fn save<E: Event>(&mut self, id: EventId<E>, event: E) -> Result<EventId<E>, Error> {
        self.events.push(serde_json::to_value(event).unwrap());
        Ok(id)
    }

    async fn load<E: Event>(&self, _id: EventId<E>) -> Result<E, Error> {
        Ok(serde_json::from_value::<E>(self.events.first().unwrap().to_owned()).unwrap())
    }

    async fn load_all<E: Event>(&self) -> Result<Vec<E>, Error> {
        Ok(self
            .events
            .iter()
            .map(|e| serde_json::from_value::<E>(e.to_owned()).unwrap())
            .collect())
    }

    async fn delete<E: Event>(&self, _id: EventId<E>) -> Result<(), Error> {
        Ok(())
    }
}

/// An event collection represents the events that occurred.
///
/// Events can be written to using an [`EventWriter`]
/// and are typically read using an [`EventReader`].
#[allow(dead_code)]
pub struct EventProvider {
    store: EventStore,
}

impl EventProvider {
    pub fn new(store: EventStore) -> Self {
        Self { store }
    }

    pub async fn send<E: Event>(&self, _event: E) -> Result<EventId<E>, Error> {
        let id = EventId {
            id: Uuid::new_v4(),
            event: Default::default(),
        };

        // self.store.save(id, event).await?;

        Ok(id)
    }
}

#[allow(dead_code)]
pub struct Subscriber<E: Event> {
    store: EventStore,
    event: PhantomData<E>,
}

impl<E: Event> Subscriber<E> {
    pub fn new(store: EventStore) -> Self {
        Self {
            store,
            event: Default::default(),
        }
    }

    pub fn on_event<Callback>(self, _callback: Callback) -> JoinHandle<Result<(), Error>>
    where
        Callback: Fn(E) -> Result<(), Error> + Send + Sync + 'static,
    {
        tokio::spawn(async move {
            info!(r#type = std::any::type_name::<E>(), "Subscribing to events");

            // loop {
            //     let events = self.store.load_all().await?;

            //     for event_id in events {
            //         let event = self.store.load(event_id).await?;
            //         callback(event)?;

            //         self.store.delete(event_id).await?;
            //     }

            //     tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            // }

            Ok(())
        })
    }
}
