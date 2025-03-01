pub mod consts;
pub mod store;
pub mod types;

use self::consts::DEFAULT_AVATARS;
use self::store::AccountStore;
use self::types::{CreateAccount, UpdateAccount};
use crate::prelude::*;
use crate::services::{ServiceFactory, ServiceProvider};
use crate::store::service::DatabaseTransaction;
use entity::account::{AccountModel};
use error_stack::{Report, Result};
use lib::error::Error;
use serenity::async_trait;
use std::sync::Arc;
use uuid::Uuid;

pub struct AccountService {
    store: Arc<StoreService>,
}

impl AccountService {
    pub fn new(store: Arc<StoreService>) -> Self {
        Self { store}
    }

    pub async fn create_if_no_exists(
        &self,
        dto: CreateAccount,
        db_tx: &mut DatabaseTransaction<'_>,
    ) -> Result<AccountModel, Error> {
        let conn = self.store.read();
        let account = AccountStore::try_find_by_address(conn, dto.address.clone()).await?;
        if let Some(account) = account {
            return Ok(account);
        }

        let account = self.create(dto, db_tx).await?;

        Ok(account)
    }

    async fn create(
        &self,
        dto: CreateAccount,
        db_tx: &mut DatabaseTransaction<'_>,
    ) -> Result<AccountModel, Error> {
        let account = AccountStore::create(db_tx.as_mut(), dto.clone()).await?;

        Ok(account)
    }

    pub async fn update(
        &self,
        id: Uuid,
        dto: UpdateAccount,
        db_tx: &mut DatabaseTransaction<'_>,
    ) -> Result<AccountModel, Error> {
        let account_updated = AccountStore::update(db_tx.as_mut(), id, dto).await?;

        Ok(account_updated)
    }

    pub fn is_default_avatar(&self, avatar: &str) -> bool {
        DEFAULT_AVATARS.contains(&avatar)
    }
}

#[async_trait]
impl ServiceFactory for AccountService {
    async fn factory(services: ServiceProvider) -> Result<Self, Error> {
        let store = services.get_service_unchecked::<StoreService>().await;

        Ok(Self { store})
    }
}
