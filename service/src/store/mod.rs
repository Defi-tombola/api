pub mod service;

pub const SHORT_STEP: &str = r#"(date_trunc('hour', "vault_share"."created_at") +
(((date_part('minute', "vault_share"."created_at")::integer / 10::integer) * 10::integer)
|| ' minutes')::interval)"#;

pub const LONG_STEP: &str = r#"date_trunc('day', "vault_share"."created_at")"#;
