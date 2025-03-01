use async_trait::async_trait;
use error_stack::Result;
use lib::error::Error;
use service::{
    prelude::{ServiceProvider, StoreService},
    transaction::service::TransactionService,
};

use crate::stream::{Subscription, Validator};

use super::subscription::ChainSubscription;

/// Implement [`Validator`] for [`ChainSubscription`]
#[derive(Clone)]
pub(crate) struct EventValidator {
    services: ServiceProvider,
}

impl EventValidator {
    pub fn new(services: ServiceProvider) -> Self {
        EventValidator { services }
    }
}

#[async_trait]
impl Validator<ChainSubscription> for EventValidator {
    async fn validate(
        &self,
        input: &<ChainSubscription as Subscription>::Item,
    ) -> Result<(), Error> {
        let store_service = self.services.get_service_unchecked::<StoreService>().await;
        let transaction_service = self
            .services
            .get_service_unchecked::<TransactionService>()
            .await;

        let pool = store_service.read();

        let transaction_hash = input.transaction_hash.unwrap();
        let log_index = input.log_index.unwrap();

        Ok(())
    }
}
