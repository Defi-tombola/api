use crate::{define_find_all_fns, define_find_optional_fns, lottery::types::{ UpdateLottery}};
use chrono::{DateTime, Utc};
use entity::lottery::{LotteryModel, LotteryStatus};
use error_stack::{Result, ResultExt};
use lib::error::Error;
use sqlx::{types::Decimal, Acquire, PgPool, Pool, Postgres, QueryBuilder};
use tracing::info;
use std::future::Future;
use uuid::Uuid;

use super::types::CreateLottery;

pub struct LotteryStore;

impl LotteryStore {
    // Define common find functions
    define_find_all_fns!(find_all, "SELECT * FROM lottery", LotteryModel);
    define_find_optional_fns!(
        find_by_id,
        try_find_by_id,
        "SELECT * FROM lottery WHERE id = $1",
        Uuid,
        LotteryModel
    );

    define_find_optional_fns!(
            find_by_uid,
            try_find_by_uid,
            "SELECT * FROM lottery WHERE uid = $1",
            String,
            LotteryModel
    );

    // Create a new lottery
    #[allow(clippy::manual_async_fn)]
    pub fn create<'a, 'c, Conn>(
        conn: Conn,
        input: CreateLottery,
    ) -> impl Future<Output = Result<LotteryModel, Error>> + Send + 'a
    where
        Conn: Acquire<'c, Database = Postgres> + Send + 'a,
    {
        async move {
            let mut conn = conn
                .acquire()
                .await
                .change_context(Error::StoreTransactionFailed)?;

            let query = r#"
                INSERT INTO lottery (
                    id, uid, name, start_date, end_date, ticket_price, fee_ticket_amount, ticket_asset, max_tickets, status, created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                RETURNING *
            "#;

            let lottery = sqlx::query_as(query)
                .bind(Uuid::new_v4()) // Generate a new UUID for the lottery
                .bind(input.uid) // Bind the UID
                .bind(input.name) // Bind the name
                .bind(input.start_date) // Bind the start date
                .bind(input.end_date) // Bind the end date
                .bind(input.ticket_price) // Bind the ticket price
                .bind(input.fee_ticket_amount) // Bind the fee ticket amount
                .bind(input.ticket_asset) // Bind the ticket asset
                .bind(input.max_tickets) // Bind the optional max tickets
                .bind(input.status) // Bind the status
                .bind(Utc::now()) // Bind the created_at timestamp
                .bind(Utc::now()) // Bind the updated_at timestamp
                .fetch_one(conn.as_mut())
                .await
                .change_context(Error::StoreInsertFailed)?;

            Ok(lottery)
        }
    }

    // Update an existing lottery
    pub fn update<'c, 'a, Conn>(
        conn: Conn,
        id: Uuid,
        input: UpdateLottery,
    ) -> impl Future<Output = Result<LotteryModel, Error>> + Send + 'a
    where
        Conn: Acquire<'c, Database = Postgres> + Send + 'a,
    {
        async move {
            let mut conn = conn
                .acquire()
                .await
                .change_context(Error::StoreTransactionFailed)?;

            let query = r#"
                UPDATE lottery
                SET
                    name = COALESCE($2, name),
                    start_date = COALESCE($3, start_date),
                    end_date = COALESCE($4, end_date),
                    ticket_price = COALESCE($5, ticket_price),
                    fee_ticket_amount = COALESCE($6, fee_ticket_amount),
                    ticket_asset = COALESCE($7, ticket_asset),
                    max_tickets = COALESCE($8, max_tickets),
                    status = COALESCE($9, status),
                    updated_at = NOW()
                WHERE id = $1
                RETURNING *
            "#;

            let lottery = sqlx::query_as(query)
                .bind(id) // Bind the lottery ID to update
                .bind(input.name) // Bind the optional name
                .bind(input.start_date) // Bind the optional start date
                .bind(input.end_date) // Bind the optional end date
                .bind(input.ticket_price) // Bind the optional ticket price
                .bind(input.fee_ticket_amount) // Bind the optional fee ticket amount
                .bind(input.ticket_asset) // Bind the optional ticket asset
                .bind(input.max_tickets) // Bind the optional max tickets
                .bind(input.status) // Bind the optional status
                .fetch_one(conn.as_mut())
                .await
                .change_context(Error::StoreUpdateFailed)?;

            Ok(lottery)
        }
    }

    // Find all lotteries by a list of IDs
    pub async fn find_all_by_ids(
        pool: &PgPool,
        ids: Vec<Uuid>,
    ) -> Result<Vec<LotteryModel>, Error> {
        if ids.is_empty() {
            return Ok(Vec::new()); // Return an empty vector if no IDs are provided
        }

        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT * FROM lottery WHERE id IN ("
        );

        // Add each ID to the query
        let mut separated = query_builder.separated(", ");
        for id in ids {
            separated.push_bind(id);
        }
        separated.push_unseparated(")");

        let lotteries = query_builder
            .build_query_as::<LotteryModel>()
            .fetch_all(pool)
            .await
            .change_context(Error::Store)?;

        Ok(lotteries)
    }
    
    pub async fn find_by_filter(pool: &PgPool, filter: LotteryFilter) -> Result<Vec<LotteryModel>, Error> {
        let mut query_builder = QueryBuilder::new("SELECT * FROM lottery");

        if let Some(featured) = filter.featured {
            query_builder.push(" WHERE featured = ");
            query_builder.push_bind(featured);
        }

        if let Some(uid) = filter.uid {
            query_builder.push(" AND uid = ");
            query_builder.push_bind(uid);
        }

        if let Some(status) = filter.status {
            query_builder.push(" AND status = ");
            query_builder.push_bind(status);
        }

        query_builder.push(" ORDER BY created_at DESC");

        let lotteries = query_builder
            .build_query_as::<LotteryModel>()
            .fetch_all(pool)
            .await
            .change_context(Error::Store)?;

        Ok(lotteries)
    }
}

#[derive(Debug, Clone, Default)]
pub struct LotteryFilter {
    pub featured: Option<bool>,
    pub uid: Option<String>,
    pub status: Option<LotteryStatus>,
}