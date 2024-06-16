mod config;
mod connection;
mod types;

pub type DbResult<T> = anyhow::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Pool error: {0}")]
    PoolError(#[from] deadpool_postgres::PoolError),
    #[error("Create Pool error: {0}")]
    CreatePoolError(#[from] deadpool_postgres::CreatePoolError),
    #[error("PostgreSQL error: {0}")]
    PgError(#[from] tokio_postgres::Error),
}
