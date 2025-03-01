use std::sync::Arc;

use error_stack::Result;

use super::types::{CreateTransaction, CreateTransactionSideEffect};
use crate::store::service::DatabaseTransaction;
use crate::transaction::store::TransactionStore;
use crate::{
    chain::types::EventContext, prelude::ServiceProvider, services::ServiceFactory,
    store::service::StoreService,
};
use entity::prelude::TransactionLogModel;
use lib::error::Error;
use serenity::async_trait;
use tracing::info;

pub struct TransactionService {
    #[allow(dead_code)]
    store: Arc<StoreService>,
}

impl TransactionService {
    pub fn new(store: Arc<StoreService>) -> Self {
        Self { store }
    }

    /// Create a transaction log with side effects
    pub async fn create(
        &self,
        dto: CreateTransaction,
        db_tx: &mut DatabaseTransaction<'_>,
    ) -> Result<TransactionLogModel, Error> {
        let transaction_log_model = TransactionStore::create(db_tx.as_mut(), dto.clone()).await?;

        let side_effects = CreateTransactionSideEffect {
            side_effects: dto.side_effects.clone(),
            transaction_log_id: transaction_log_model.id,
        };

        let side_effects = TransactionStore::create_side_effects(db_tx, side_effects).await?;

        info!(
            "Created chain transaction log {:?} with {} side effect(s)",
            transaction_log_model.transaction_hash,
            side_effects.len()
        );

        Ok(transaction_log_model)
    }

    pub async fn create_without_side_effects(
        &self,
        context: EventContext,
        db_tx: &mut DatabaseTransaction<'_>,
    ) -> Result<TransactionLogModel, Error> {
        let dto = CreateTransaction {
            context: context.clone(),
            side_effects: vec![],
            created_at: context.triggered_at,
        };

        let transaction_log = TransactionStore::create(db_tx.as_mut(), dto).await?;

        Ok(transaction_log)
    }
}

#[async_trait]
impl ServiceFactory for TransactionService {
    async fn factory(services: ServiceProvider) -> Result<Self, Error> {
        let store = services.get_service_unchecked::<StoreService>().await;

        Ok(Self { store })
    }
}
