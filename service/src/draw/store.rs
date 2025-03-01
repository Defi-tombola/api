use crate::{define_find_all_fns, define_find_optional_fns, draw::types::{CreateDraw, UpdateDraw}};
use chrono::{DateTime, Utc};
use entity::draw::{DrawModel, DrawStatus};
use error_stack::{Result, ResultExt};
use lib::error::Error;
use sqlx::{types::Uuid, Acquire, PgPool, Postgres, QueryBuilder};
use std::future::Future;

pub struct DrawStore;

impl DrawStore {
    // Define common find functions
    define_find_all_fns!(find_all, "SELECT * FROM draw", DrawModel);
    define_find_optional_fns!(
        find_by_id,
        try_find_by_id,
        "SELECT * FROM draw WHERE id = $1",
        Uuid,
        DrawModel
    );
    define_find_optional_fns!(
            find_by_lottery_id,
            try_find_by_lottery_id,
            "SELECT * FROM draw WHERE lottery_id = $1",
            Uuid,
            DrawModel
        );
        
        define_find_optional_fns!(
            find_by_transaction_hash,
            try_find_by_transaction_hash,
            "SELECT * FROM draw WHERE transaction_hash = $1",
            String,
            DrawModel
        );
    define_find_all_fns!(
        find_draw_by_winner,
        "SELECT * FROM draw WHERE winner = $1",
        Uuid,
        DrawModel
    );
        
    // Create a new draw
    #[allow(clippy::manual_async_fn)]
    pub fn create<'a, 'c, Conn>(
        conn: Conn,
        input: CreateDraw,
    ) -> impl Future<Output = Result<DrawModel, Error>> + Send + 'a
    where
        Conn: Acquire<'c, Database = Postgres> + Send + 'a,
    {
        async move {
            let mut conn = conn
                .acquire()
                .await
                .change_context(Error::StoreTransactionFailed)?;

            let query = r#"
                INSERT INTO draw (
                    id, lottery_id, status, created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5)
                RETURNING *
            "#;

            let draw = sqlx::query_as(query)
                .bind(Uuid::new_v4()) // Generate a new UUID for the draw
                .bind(input.lottery_id) // Bind the lottery ID
                .bind(input.status) // Bind the status
                .bind(Utc::now()) // Bind the created_at timestamp
                .bind(Utc::now()) // Bind the updated_at timestamp
                .fetch_one(conn.as_mut())
                .await
                .change_context(Error::StoreInsertFailed)?;

            Ok(draw)
        }
    }

    // Update an existing draw
    pub fn update<'c, 'a, Conn>(
        conn: Conn,
        id: Uuid,
        input: UpdateDraw,
    ) -> impl Future<Output = Result<DrawModel, Error>> + Send + 'a
    where
        Conn: Acquire<'c, Database = Postgres> + Send + 'a,
    {
        async move {
            let mut conn = conn
                .acquire()
                .await
                .change_context(Error::StoreTransactionFailed)?;

            let query = r#"
                UPDATE draw
                SET
                    draw_date = COALESCE($2, draw_date),
                    winner = COALESCE($3, winner),
                    transaction_hash = COALESCE($4, transaction_hash),
                    status = COALESCE($5, status),
                    updated_at = NOW()
                WHERE id = $1
                RETURNING *
            "#;

            let draw = sqlx::query_as(query)
                .bind(id) // Bind the draw ID to update
                .bind(input.draw_date) // Bind the optional draw date
                .bind(input.winner) // Bind the optional winning ticket ID
                .bind(input.transaction_hash) // Bind the optional transaction hash
                .bind(input.status) // Bind the optional status
                .fetch_one(conn.as_mut())
                .await
                .change_context(Error::StoreUpdateFailed)?;

            Ok(draw)
        }
    }

    // Find all draws by a list of IDs
    pub async fn find_all_by_ids(
        pool: &PgPool,
        ids: Vec<Uuid>,
    ) -> Result<Vec<DrawModel>, Error> {
        if ids.is_empty() {
            return Ok(Vec::new()); // Return an empty vector if no IDs are provided
        }

        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT * FROM draw WHERE id IN ("
        );

        // Add each ID to the query
        let mut separated = query_builder.separated(", ");
        for id in ids {
            separated.push_bind(id);
        }
        separated.push_unseparated(")");

        // Execute the query
        let draws = query_builder
            .build_query_as::<DrawModel>()
            .fetch_all(pool)
            .await
            .change_context(Error::Store)?;

        Ok(draws)
    }
}