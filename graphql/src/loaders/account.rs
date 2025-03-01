use async_graphql::dataloader::Loader;
use entity::prelude::AccountModel;
use lib::error::Error;
use service::account::store::AccountStore;
use service::store::service::StoreService;
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

#[derive(Clone)]
pub struct AccountLoader {
    store: Arc<StoreService>,
}

impl AccountLoader {
    pub fn new(store: Arc<StoreService>) -> AccountLoader {
        AccountLoader { store }
    }
}

impl Loader<Uuid> for AccountLoader {
    type Value = AccountModel;
    type Error = Arc<Error>;

    async fn load(
        &self,
        keys: &[Uuid],
    ) -> async_graphql::Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let pool = self.store.read();
        let results = match AccountStore::find_all_by_ids(pool, keys.to_vec()).await {
            Ok(results) => results,
            Err(e) => {
                tracing::error!("Failed to load accounts. Error: {e:?}");
                return Err(Arc::new(Error::Store));
            }
        };

        let mut map = HashMap::new();

        for result in results {
            map.insert(result.id, result);
        }

        Ok(map)
    }
}
