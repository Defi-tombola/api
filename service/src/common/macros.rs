// src/lib.rs (or src/main.rs)
#[allow(unused_imports)]
use error_stack::{Result, ResultExt};
#[allow(unused_imports)]
use sqlx::Postgres;

#[macro_export]
macro_rules! define_find_optional_fns {
    ($find_fn_name:ident, $try_fn_name:ident, $query:expr, $param_type:ty, $return_type:ty) => {
        pub fn $try_fn_name<'a, 'c, Connection>(
            conn: Connection,
            param: $param_type,
        ) -> impl Future<Output = Result<Option<$return_type>, Error>> + Send + 'a
        where
            Connection: Acquire<'c, Database = Postgres> + Send + 'a,
        {
            async move {
                let mut conn = conn.acquire().await.change_context(Error::Store)?;

                let query = sqlx::query_as($query)
                    .bind(param)
                    .fetch_optional(&mut *conn)
                    .await
                    .change_context(Error::Store)?;
                Ok(query)
            }
        }

        pub fn $find_fn_name<'a, 'c, Connection>(
            conn: Connection,
            param: $param_type,
        ) -> impl Future<Output = Result<$return_type, Error>> + Send + 'a
        where
            Connection: Acquire<'c, Database = Postgres> + Send + 'a,
        {
            async move {
                let mut conn = conn.acquire().await.change_context(Error::Store)?;

                let result = sqlx::query_as($query)
                    .bind(param)
                    .fetch_optional(&mut *conn)
                    .await
                    .change_context(Error::Store)?;

                if result.is_none() {
                    return Err(error_stack::Report::new(lib::error::Error::NotFound));
                }

                Ok(result.unwrap())
            }
        }
    };
}

#[macro_export]
macro_rules! define_find_all_fns {
    ($find_fn_name:ident, $query:expr, $param_type:ty, $return_type:ty) => {
        pub fn $find_fn_name<'a, 'c, Connection>(
            conn: Connection,
            param: $param_type,
        ) -> impl Future<Output = Result<Vec<$return_type>, Error>> + Send + 'a
        where
            Connection: Acquire<'c, Database = Postgres> + Send + 'a,
        {
            async move {
                let mut conn = conn.acquire().await.change_context(Error::Store)?;
                let query = sqlx::query_as($query)
                    .bind(param)
                    .fetch_all(&mut *conn)
                    .await
                    .change_context(Error::Store)?;
                Ok(query)
            }
        }
    };

    ($find_fn_name:ident, $query:expr, $return_type:ty) => {
        pub fn $find_fn_name<'a, 'c, Connection>(
            conn: Connection,
        ) -> impl Future<Output = Result<Vec<$return_type>, Error>> + Send + 'a
        where
            Connection: Acquire<'c, Database = Postgres> + Send + 'a,
        {
            async move {
                let mut conn = conn.acquire().await.change_context(Error::Store)?;
                let query = sqlx::query_as($query)
                    .fetch_all(&mut *conn)
                    .await
                    .change_context(Error::Store)?;
                Ok(query)
            }
        }
    };
}

#[macro_export]
macro_rules! build_in_query {
    ($base_query:expr, $params:expr) => {{
        let values_str = $params
            .iter()
            .map(|val| format!("'{}'", val.to_string()))
            .collect::<Vec<String>>()
            .join(", ");

        format!("{} IN ({})", $base_query, values_str)
    }};
}

#[macro_export]
/// Builds and wraps query to count rows
/// Returns query_scalar with i64 as return type
/// Binding values can be passed on query build.
/// Example:
/// ```rust
/// let query = build_count_query!("SELECT * FROM table WHERE id = $1", id);
/// ```
///
/// Will be converted to:
/// ```sql
/// SELECT COUNT(*) AS "count" FROM (
///     SELECT * FROM table WHERE id = $1
/// ) as "inner_table"
/// ```
macro_rules! build_count_query {
    ($base_query:expr) => {{
        sqlx::query_scalar::<_, i64>(
            format!(
                r#"SELECT COUNT(*) AS "count" FROM ({}) AS "inner_table""#,
                $base_query
            )
            .as_str(),
        )
    }};
}

#[macro_export]
/// Builds query with page and page size
/// Returns query (as SQL) with limit and offset
/// Example (page: 200, page_size: 10):
/// ```rust
/// let query = build_paginated_query!("SELECT * FROM table", 20, 10);
/// ```
///
/// Will return:
/// ```sql
/// SELECT * FROM table LIMIT 20 OFFSET 10
/// ```
macro_rules! build_paginated_query {
    ($base_query:expr, $page:expr, $page_size:expr) => {{
        let limit = format!(" LIMIT {}", $page_size);
        let offset = format!(" OFFSET {}", ($page * $page_size) as i64);

        format!("{} {} {}", $base_query, limit, offset)
    }};
}
