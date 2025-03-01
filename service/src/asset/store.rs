use chrono::{DateTime, Utc};
use entity::asset::AssetModel;
use error_stack::{Result, ResultExt};
use lib::error::Error;
use sqlx::{types::Uuid, Acquire, PgPool, Postgres, QueryBuilder};
use std::future::Future;

use crate::{define_find_all_fns, define_find_optional_fns};

pub struct AssetStore;

impl AssetStore {
    // Define common find functions
    define_find_all_fns!(find_all, "SELECT * FROM asset", AssetModel);
    define_find_optional_fns!(
        find_by_id,
        try_find_by_id,
        "SELECT * FROM asset WHERE id = $1",
        Uuid,
        AssetModel
    );
    define_find_optional_fns!(
        find_by_address,
        try_find_by_address,
        "SELECT * FROM asset WHERE LOWER(address) = LOWER($1)",
        String,
        AssetModel
    );
    
    pub async fn find_all_by_ids(
        pool: &PgPool,
        ids: Vec<Uuid>,
    ) -> Result<Vec<AssetModel>, Error> {
        if ids.is_empty() {
            return Ok(Vec::new()); // Return an empty vector if no IDs are provided
        }

        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT * FROM asset WHERE id IN ("
        );

        let mut separated = query_builder.separated(", ");
        for id in ids {
            separated.push_bind(id);
        }
        separated.push_unseparated(")");

        let assets = query_builder
            .build_query_as::<AssetModel>()
            .fetch_all(pool)
            .await
            .change_context(Error::Store)?;

        Ok(assets)
    }
}