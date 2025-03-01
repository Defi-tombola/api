use crate::chain_state::types::{CreateChainState, UpdateChainState};
use crate::define_find_optional_fns;
use entity::chain_state::ChainStateModel;
use error_stack::{Result, ResultExt};
use lib::error::Error;
use sqlx::{Acquire, Postgres};
use std::future::Future;
use uuid::Uuid;

pub struct ChainStateStore;

impl ChainStateStore {
    define_find_optional_fns!(
        find_by_chain_name,
        try_find_by_chain_name,
        "SELECT * FROM chain_state WHERE chain = $1",
        String,
        ChainStateModel
    );
    define_find_optional_fns!(
        find_by_id,
        try_find_by_id,
        "SELECT * FROM chain_state WHERE id = $1",
        Uuid,
        ChainStateModel
    );

    #[allow(clippy::manual_async_fn)]
    pub fn create<'a, 'c, Conn>(
        conn: Conn,
        input: CreateChainState,
    ) -> impl Future<Output = Result<ChainStateModel, Error>> + Send + 'a
    where
        Conn: Acquire<'c, Database = Postgres> + Send + 'a,
    {
        async move {
            let mut conn = conn
                .acquire()
                .await
                .change_context(Error::StoreTransactionFailed)?;

            let query = r#"
                INSERT INTO chain_state
                (id, chain, value, updated_at)
                VALUES ($1, $2, $3, $4)
                RETURNING *
            "#;

            let result = sqlx::query_as(query)
                .bind(Uuid::new_v4())
                .bind(input.chain)
                .bind(input.value)
                .bind(input.updated_at)
                .fetch_one(conn.as_mut())
                .await
                .change_context(Error::StoreInsertFailed)?;

            Ok(result)
        }
    }
    #[allow(clippy::manual_async_fn)]
    pub fn update_by_chain_name<'a, 'c, Conn>(
        conn: Conn,
        chain_name: String,
        input: UpdateChainState,
    ) -> impl Future<Output = Result<ChainStateModel, Error>> + Send + 'a
    where
        Conn: Acquire<'c, Database = Postgres> + Send + 'a,
    {
        async move {
            let mut conn = conn
                .acquire()
                .await
                .change_context(Error::StoreTransactionFailed)?;

            let query = r#"
                UPDATE chain_state
                SET value = COALESCE($2, value),
                updated_at = COALESCE($3, updated_at)
                WHERE chain = $1
                RETURNING *
            "#;

            let result = sqlx::query_as(query)
                .bind(chain_name)
                .bind(input.value)
                .bind(input.updated_at)
                .fetch_one(conn.as_mut())
                .await
                .change_context(Error::StoreUpdateFailed)?;

            Ok(result)
        }
    }
}
