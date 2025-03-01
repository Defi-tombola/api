use crate::account::consts::DEFAULT_AVATARS;
use crate::account::types::{CreateAccount, UpdateAccount};
use crate::common::types::ChartDataset;
use crate::{build_in_query, define_find_all_fns, define_find_optional_fns};
use chrono::{DateTime, Utc};
use entity::account::AccountModel;
use error_stack::{Result, ResultExt};
use lib::error::Error;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use rust_decimal::prelude::{ToPrimitive, Zero};
use sqlx::{types::Decimal, Acquire, PgPool, Postgres, QueryBuilder};
use std::future::Future;
use std::ops::{Div, Mul, Sub};
use uuid::Uuid;

pub struct AccountStore;

impl AccountStore {
    define_find_all_fns!(find_all, "SELECT * FROM account", AccountModel);
    define_find_optional_fns!(
        find_by_address,
        try_find_by_address,
        "SELECT * FROM account WHERE LOWER(address) = LOWER($1)",
        String,
        AccountModel
    );
    define_find_optional_fns!(
        find_by_id,
        try_find_by_id,
        "SELECT * FROM account WHERE id = $1",
        Uuid,
        AccountModel
    );

    pub fn get_default_avatar() -> Option<String> {
        Vec::from(DEFAULT_AVATARS)
            .choose(&mut thread_rng())
            .map(|i| (*i).to_string())
    }

    #[allow(clippy::manual_async_fn)]
    pub fn create<'a, 'c, Conn>(
        conn: Conn,
        input: CreateAccount,
    ) -> impl Future<Output = Result<AccountModel, Error>> + Send + 'a
    where
        Conn: Acquire<'c, Database = Postgres> + Send + 'a,
    {
        async move {
            let mut conn = conn
                .acquire()
                .await
                .change_context(Error::StoreTransactionFailed)?;
    
            let query = r#"
                INSERT INTO account (
                    id, address, created_at, updated_at
                )
                VALUES ($1, $2, $3, $4)
                RETURNING *
            "#;
    
            let account = sqlx::query_as(query)
                .bind(Uuid::new_v4()) // Generate a new UUID for the account
                .bind(input.address.clone()) // Bind the address from the input
                .bind(input.created_at) // Bind the created_at from the input
                .bind(input.created_at) // Bind the updated_at from the input
                .fetch_one(conn.as_mut())
                .await
                .change_context(Error::StoreInsertFailed)?;
    
            Ok(account)
        }
    }
    
    pub fn update<'c, 'a, Conn>(
        conn: Conn,
        id: Uuid,
        account: UpdateAccount,
    ) -> impl Future<Output = Result<AccountModel, Error>> + Send + 'a
    where
        Conn: Acquire<'c, Database = Postgres> + Send + 'a,
    {
        async move {
            let mut conn = conn
                .acquire()
                .await
                .change_context(Error::StoreTransactionFailed)?;
    
            let query = r#"
                UPDATE account
                SET
                    avatar = COALESCE($2, avatar),
                    name = COALESCE($3, name),
                    twitter = COALESCE($4, twitter),
                    updated_at = NOW()
                WHERE id = $1
                RETURNING *
            "#;
    
            let account = sqlx::query_as(query)
                .bind(id) // Bind the account ID to update
                .bind(account.avatar) // Bind the optional avatar
                .bind(account.name) // Bind the optional name
                .bind(account.twitter) // Bind the optional Twitter handle
                .fetch_one(conn.as_mut())
                .await
                .change_context(Error::StoreUpdateFailed)?;
    
            Ok(account)
        }
    }
    
    pub async fn find_all_by_ids(
        pool: &PgPool,
        ids: Vec<Uuid>,
    ) -> Result<Vec<AccountModel>, Error> {
        if ids.is_empty() {
            return Ok(Vec::new()); // Return an empty vector if no IDs are provided
        }
    
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT * FROM account WHERE id IN ("
        );
    
        // Add each ID to the query
        let mut separated = query_builder.separated(", ");
        for id in ids {
            separated.push_bind(id);
        }
        separated.push_unseparated(")");
    
        // Execute the query
        let accounts = query_builder
            .build_query_as::<AccountModel>()
            .fetch_all(pool)
            .await
            .change_context(Error::Store)?;
        
        Ok(accounts)
    }
}
