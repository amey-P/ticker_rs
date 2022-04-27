use crate::tickers::POOL;

use redis;

pub fn default_connection() -> redis::Connection {
    *POOL.clone().get().unwrap()
}
