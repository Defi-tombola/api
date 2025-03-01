use async_graphql::dataloader::Loader;
use entity::asset::AssetModel;
use error_stack::Report;
use lib::error::Error;
use service::asset::store::AssetStore;
use service::store::service::StoreService;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct AssetLoader {
    store: Arc<StoreService>,
}

impl AssetLoader {
    pub fn new(store: Arc<StoreService>) -> Self {
        AssetLoader { store }
    }
}

impl Loader<Uuid> for AssetLoader {
    type Value = AssetModel;
    type Error = Arc<Report<Error>>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let pool = self.store.read();
        let results = AssetStore::find_all_by_ids(pool, keys.to_vec())
            .await
            .map_err(Arc::new)?;

        let mut map = HashMap::new();
        for result in results {
            map.insert(result.id, result);
        }

        Ok(map)
    }
}
