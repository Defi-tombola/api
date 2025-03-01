use crate::{define_find_all_fns, define_find_optional_fns, ticket::types::{CreateTicket, UpdateTicket}};
use chrono::{DateTime, Utc};
use entity::ticket::{TicketModel};
use error_stack::{Result, ResultExt};
use lib::error::Error;
use sqlx::{types::Uuid, Acquire, PgPool, Postgres, QueryBuilder};
use std::future::Future;

pub struct TicketStore;

impl TicketStore {
    // Define common find functions
    define_find_all_fns!(find_all, "SELECT * FROM ticket", TicketModel);
    define_find_optional_fns!(
        find_by_id,
        try_find_by_id,
        "SELECT * FROM ticket WHERE id = $1",
        Uuid,
        TicketModel
    );
    
    define_find_all_fns!(
        find_last_bought_tickets,
        "SELECT * FROM ticket ORDER BY purchased_at DESC LIMIT $1",
        i32,
        TicketModel
    );
    
    define_find_all_fns!(
        find_last_bought_tickets_by_lottery_id,
        "SELECT * FROM ticket WHERE lottery_id = $1 ORDER BY purchased_at DESC",
        Uuid,
        TicketModel
    );
    
    define_find_all_fns!(
        find_by_lottery_id,
        "SELECT * FROM ticket WHERE lottery_id = $1",
        Uuid,
        TicketModel
    );
    

    // Create a new ticket
    #[allow(clippy::manual_async_fn)]
    pub fn create<'a, 'c, Conn>(
        conn: Conn,
        input: CreateTicket,
    ) -> impl Future<Output = Result<TicketModel, Error>> + Send + 'a
    where
        Conn: Acquire<'c, Database = Postgres> + Send + 'a,
    {
        async move {
            let mut conn = conn
                .acquire()
                .await
                .change_context(Error::StoreTransactionFailed)?;

            let query = r#"
                INSERT INTO ticket (
                    id, lottery_id, account_id, ticket_price, ticket_asset, amount, transaction_hash, purchased_at, created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                RETURNING *
            "#;

            let ticket = sqlx::query_as(query)
                .bind(Uuid::new_v4()) // Generate a new UUID for the ticket
                .bind(input.lottery_id) // Bind the lottery ID
                .bind(input.account_id) // Bind the account ID
                .bind(input.ticket_price) // Bind the ticket price
                .bind(input.ticket_asset) // Bind the ticket asset
                .bind(input.amount) // Bind the number of tickets
                .bind(input.transaction_hash) // Bind the transaction hash
                .bind(input.purchased_at) // Bind the purchase timestamp
                .bind(Utc::now()) // Bind the created_at timestamp
                .bind(Utc::now()) // Bind the updated_at timestamp
                .fetch_one(conn.as_mut())
                .await
                .change_context(Error::StoreInsertFailed)?;

            Ok(ticket)
        }
    }

    // Update an existing ticket
    pub fn update<'c, 'a, Conn>(
        conn: Conn,
        id: Uuid,
        input: UpdateTicket,
    ) -> impl Future<Output = Result<TicketModel, Error>> + Send + 'a
    where
        Conn: Acquire<'c, Database = Postgres> + Send + 'a,
    {
        async move {
            let mut conn = conn
                .acquire()
                .await
                .change_context(Error::StoreTransactionFailed)?;

            let query = r#"
                UPDATE ticket
                SET
                    lottery_id = COALESCE($2, lottery_id),
                    account_id = COALESCE($3, account_id),
                    amount = COALESCE($4, amount),
                    purchased_at = COALESCE($5, purchased_at),
                    updated_at = NOW()
                WHERE id = $1
                RETURNING *
            "#;

            let ticket = sqlx::query_as(query)
                .bind(id) // Bind the ticket ID to update
                .bind(input.lottery_id) // Bind the optional lottery ID
                .bind(input.account_id) // Bind the optional account ID
                .bind(input.amount) // Bind the optional ticket number
                .bind(input.purchased_at) // Bind the optional purchase timestamp
                .fetch_one(conn.as_mut())
                .await
                .change_context(Error::StoreUpdateFailed)?;

            Ok(ticket)
        }
    }

    // Find all tickets by a list of IDs
    pub async fn find_all_by_ids(
        pool: &PgPool,
        ids: Vec<Uuid>,
    ) -> Result<Vec<TicketModel>, Error> {
        if ids.is_empty() {
            return Ok(Vec::new()); // Return an empty vector if no IDs are provided
        }

        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT * FROM ticket WHERE id IN ("
        );

        // Add each ID to the query
        let mut separated = query_builder.separated(", ");
        for id in ids {
            separated.push_bind(id);
        }
        separated.push_unseparated(")");

        // Execute the query
        let tickets = query_builder
            .build_query_as::<TicketModel>()
            .fetch_all(pool)
            .await
            .change_context(Error::Store)?;

        Ok(tickets)
    }
    
    pub async fn find_by_lottery_id_and_account_id(
        pool: &PgPool,
        lottery_id: Uuid,
        account_id: Uuid,
    ) -> Result<Option<TicketModel>, Error> {
        let query = r#"
            SELECT * FROM ticket WHERE lottery_id = $1 AND account_id = $2
            LIMIT 1
            "#;

        let tickets = sqlx::query_as(query)
            .bind(lottery_id)
            .bind(account_id)
            .fetch_optional(pool)
            .await
            .change_context(Error::Store)?;
        
        Ok(tickets)
    }
    
    pub async fn find_by_address(
        pool: &PgPool,
        address: String,
    ) -> Result<Vec<TicketModel>, Error> {
        let query = r#"
            SELECT *
            FROM ticket
            INNER JOIN account ON ticket.account_id = account.id
            WHERE LOWER(account.address) = LOWER($1)
            "#;

        let tickets = sqlx::query_as(query)
            .bind(address)
            .fetch_all(pool)
            .await
            .change_context(Error::Store)?;
        
        Ok(tickets)
    }
}