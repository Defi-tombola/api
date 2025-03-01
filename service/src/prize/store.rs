use crate::{define_find_all_fns, define_find_optional_fns, prize::types::{CreatePrize, UpdatePrize}};
use chrono::{DateTime, Utc};
use entity::prize::{PrizeModel, PrizeStatus};
use error_stack::{Result, ResultExt};
use lib::error::Error;
use sqlx::{types::{Decimal, Uuid}, Acquire, PgPool, Postgres, QueryBuilder};
use std::future::Future;

pub struct PrizeStore;

impl PrizeStore {
    // Define common find functions
    define_find_all_fns!(find_all, "SELECT * FROM prize", PrizeModel);
    define_find_optional_fns!(
        find_by_id,
        try_find_by_id,
        "SELECT * FROM prize WHERE id = $1",
        Uuid,
        PrizeModel
    );
    define_find_optional_fns!(
            find_by_lottery_id,
            try_find_by_lottery_id,
            "SELECT * FROM prize WHERE lottery_id = $1",
            Uuid,
            PrizeModel
        );


    // Create a new prize
    #[allow(clippy::manual_async_fn)]
    pub fn create<'a, 'c, Conn>(
        conn: Conn,
        input: CreatePrize,
    ) -> impl Future<Output = Result<PrizeModel, Error>> + Send + 'a
    where
        Conn: Acquire<'c, Database = Postgres> + Send + 'a,
    {
        async move {
            let mut conn = conn
                .acquire()
                .await
                .change_context(Error::StoreTransactionFailed)?;

            let query = r#"
                INSERT INTO prize (
                    id, lottery_id, prize_asset, value, status, created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING *
            "#;

            let prize = sqlx::query_as(query)
                .bind(Uuid::new_v4()) // Generate a new UUID for the prize
                .bind(input.lottery_id) // Bind the lottery ID
                .bind(input.prize_asset) // Bind the prize asset
                .bind(input.value) // Bind the prize value
                .bind(input.status) // Bind the status
                .bind(Utc::now()) // Bind the created_at timestamp
                .bind(Utc::now()) // Bind the updated_at timestamp
                .fetch_one(conn.as_mut())
                .await
                .change_context(Error::StoreInsertFailed)?;

            Ok(prize)
        }
    }

    // Update an existing prize
    pub fn update<'c, 'a, Conn>(
        conn: Conn,
        id: Uuid,
        input: UpdatePrize,
    ) -> impl Future<Output = Result<PrizeModel, Error>> + Send + 'a
    where
        Conn: Acquire<'c, Database = Postgres> + Send + 'a,
    {
        async move {
            let mut conn = conn
                .acquire()
                .await
                .change_context(Error::StoreTransactionFailed)?;

            let query = r#"
                UPDATE prize
                SET
                    lottery_id = COALESCE($2, lottery_id),
                    prize_asset = COALESCE($3, prize_asset),
                    value = COALESCE($4, value),
                    status = COALESCE($5, status),
                    updated_at = NOW()
                WHERE id = $1
                RETURNING *
            "#;

            let prize = sqlx::query_as(query)
                .bind(id) // Bind the prize ID to update
                .bind(input.lottery_id) // Bind the optional lottery ID
                .bind(input.prize_asset) // Bind the optional prize asset
                .bind(input.value) // Bind the optional prize value
                .bind(input.status) // Bind the optional status
                .fetch_one(conn.as_mut())
                .await
                .change_context(Error::StoreUpdateFailed)?;

            Ok(prize)
        }
    }

    // Find all prizes by a list of IDs
    pub async fn find_all_by_ids(
        pool: &PgPool,
        ids: Vec<Uuid>,
    ) -> Result<Vec<PrizeModel>, Error> {
        if ids.is_empty() {
            return Ok(Vec::new()); // Return an empty vector if no IDs are provided
        }

        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT * FROM prize WHERE id IN ("
        );

        // Add each ID to the query
        let mut separated = query_builder.separated(", ");
        for id in ids {
            separated.push_bind(id);
        }
        separated.push_unseparated(")");

        // Execute the query
        let prizes = query_builder
            .build_query_as::<PrizeModel>()
            .fetch_all(pool)
            .await
            .change_context(Error::Store)?;

        Ok(prizes)
    }
}