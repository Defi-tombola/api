use crate::chain::traits::string::ToHexString;
use crate::store::service::DatabaseTransaction;
use crate::transaction::types::{CreateTransaction, CreateTransactionSideEffect};
use crate::{define_find_all_fns, define_find_optional_fns};
use entity::prelude::{TransactionLogModel, TransactionLogSideEffectModel};
use error_stack::{Result, ResultExt};
use ethers::types::{H256, U256};
use lib::error::Error;
use sqlx::{query_as, Acquire, PgPool, Postgres};
use std::future::Future;
use uuid::Uuid;

pub struct TransactionStore;

impl TransactionStore {
    define_find_all_fns!(
        find_all_transaction_log,
        "SELECT * FROM transaction_log",
        TransactionLogModel
    );
    define_find_all_fns!(
        find_all_transaction_log_side_effect,
        "SELECT * FROM transaction_log_side_effect",
        TransactionLogSideEffectModel
    );

    define_find_optional_fns!(
        find_by_transaction_hash,
        try_find_by_transaction_hash,
        "SELECT * FROM transaction_log WHERE transaction_hash = $1",
        String,
        TransactionLogModel
    );

    #[allow(clippy::manual_async_fn)]
    pub fn create<'a, 'c, Conn>(
        conn: Conn,
        input: CreateTransaction,
    ) -> impl Future<Output = Result<TransactionLogModel, Error>> + Send + 'a
    where
        Conn: Acquire<'c, Database = Postgres> + Send + 'a,
    {
        async move {
            let mut conn = conn
                .acquire()
                .await
                .change_context(Error::StoreTransactionFailed)?;

            let chain = input.context.chain.clone();
            let src_address = input.context.src_address.to_hex_string();
            let block_number = input.context.block_number.as_u32() as i64;
            let transaction_hash = format!("{:#x}", input.context.transaction_hash);
            let log_index = input.context.log_index.as_u32() as i32;
            let created_at = input.created_at;

            let transaction_log = query_as(
                r#"
                INSERT INTO transaction_log (
                id, chain, address,
                block_number, transaction_hash,
                log_index, created_at
                )
                VALUES (
                $1, $2, $3,
                $4, $5, $6, $7
                )
                RETURNING *
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(chain)
            .bind(src_address)
            .bind(block_number)
            .bind(transaction_hash)
            .bind(log_index)
            .bind(created_at)
            .fetch_one(conn.as_mut())
            .await
            .unwrap();

            Ok(transaction_log)
        }
    }

    #[allow(clippy::manual_async_fn)]
    pub fn create_side_effects<'a, 'c, Conn>(
        conn: Conn,
        input: CreateTransactionSideEffect,
    ) -> impl Future<Output = Result<Vec<TransactionLogSideEffectModel>, Error>> + Send + 'a
    where
        Conn: Acquire<'c, Database = Postgres> + Send + 'a,
    {
        async move {
            let mut conn = conn
                .acquire()
                .await
                .change_context(Error::StoreTransactionFailed)?;
            let mut side_effects: Vec<TransactionLogSideEffectModel> = Vec::new();

            for side_effect in input.side_effects.iter() {
                let transaction_log_side_effect = query_as(
                    r#"
                    INSERT INTO transaction_log_side_effect (id, transaction_log_id, entity_id, entity_type)
                    VALUES ($1, $2, $3, $4)
                    RETURNING *
                    "#,
                )
                    .bind(Uuid::new_v4())
                    .bind(input.transaction_log_id)
                    .bind(side_effect.entity_id.clone())
                    .bind(side_effect.entity_type.clone())
                    .fetch_one(conn.as_mut())
                    .await
                    .map_err(|_e| Error::Store)?;

                side_effects.push(transaction_log_side_effect);
            }

            Ok(side_effects)
        }
    }

    pub async fn get_duplicates(
        pool: &PgPool,
    ) -> Result<Vec<TransactionLogSideEffectModel>, Error> {
        let query = r#"
        SELECT * FROM transaction_log_side_effect WHERE transaction_log_id IN ( SELECT id FROM (
                SELECT id,
                ROW_NUMBER() OVER(PARTITION BY transaction_hash, log_index ORDER BY id ASC) AS row
                FROM transaction_log
            ) txs
            WHERE txs.ROW > 1
        );"#;

        let side_effects = query_as(query)
            .fetch_all(pool)
            .await
            .change_context(Error::Store)?;

        Ok(side_effects)
    }

    pub async fn delete_transaction_log_by_id(
        db_tx: &mut DatabaseTransaction<'_>,
        id: Uuid,
    ) -> Result<(), Error> {
        let db_tx = db_tx
            .acquire()
            .await
            .change_context(Error::StoreTransactionFailed)?;

        let query = r#"
        DELETE FROM transaction_log WHERE id = $1
        "#;

        sqlx::query(query)
            .bind(id)
            .execute(db_tx)
            .await
            .change_context(Error::Store)?;

        Ok(())
    }

    pub async fn delete_transaction_log_side_effect_by_id(
        db_tx: &mut DatabaseTransaction<'_>,
        id: Uuid,
    ) -> Result<(), Error> {
        let db_tx = db_tx
            .acquire()
            .await
            .change_context(Error::StoreTransactionFailed)?;

        let query = r#"
        DELETE FROM transaction_log_side_effect WHERE id = $1
        "#;

        sqlx::query(query)
            .bind(id)
            .execute(db_tx)
            .await
            .change_context(Error::Store)?;

        Ok(())
    }

    pub async fn try_find_by_hash_and_log_index(
        pool: &PgPool,
        hash: H256,
        log_index: U256,
    ) -> Result<Option<TransactionLogModel>, Error> {
        let query = r#"
            SELECT * 
            FROM transaction_log
            WHERE transaction_hash = $1
            AND log_index = $2
        "#;

        let transaction_log = query_as(query)
            .bind(hash.to_hex_string())
            .bind(log_index.as_u32() as i32)
            .fetch_optional(pool)
            .await
            .change_context(Error::Store)?;

        Ok(transaction_log)
    }
}
