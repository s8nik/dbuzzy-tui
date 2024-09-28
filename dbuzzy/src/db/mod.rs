// @todo:
#![allow(dead_code)]

mod conn;
pub mod tree;

pub use conn::{ConnConfig, PgPool};

pub type DbResult<T> = anyhow::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Pool error: {0}")]
    PgPool(#[from] deadpool_postgres::PoolError),
    #[error("Create Pool error: {0}")]
    CreatePgPool(#[from] deadpool_postgres::CreatePoolError),
    #[error("PostgreSQL error: {0}")]
    Postgres(#[from] tokio_postgres::Error),
}
