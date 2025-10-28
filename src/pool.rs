use crate::error::PoolError;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = diesel::r2d2::PooledConnection<ConnectionManager<PgConnection>>;

pub fn create_pool(database_url: &str, worker_count: usize) -> Result<DbPool, PoolError> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .max_size((worker_count + 2) as u32)
        .connection_timeout(std::time::Duration::from_secs(5))
        .test_on_check_out(true)
        .build(manager)
        .map_err(PoolError::from)
}
