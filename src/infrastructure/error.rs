use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error(transparent)]
    PoolError(#[from] r2d2::Error),
    #[error(transparent)]
    SqlError(#[from] rusqlite::Error),
}
