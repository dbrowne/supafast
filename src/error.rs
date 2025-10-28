use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkerError {
    #[error("Database connection error")]
    ConnectionError(#[from] diesel::r2d2::Error),

    #[error("Database query error")]
    DatabaseError(#[from] diesel::result::Error),

    #[error("Validation error: {0}")]
    ValidationError(&'static str),

    #[error("Processing error")]
    ProcessingError,
}

#[derive(Error, Debug)]
pub enum PoolError {
    #[error("Failed to create connection pool")]
    CreationError(#[from] diesel::r2d2::Error),
}
