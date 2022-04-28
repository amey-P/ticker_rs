use redis;
use r2d2;

#[cfg(any(test, debug_assertions))]
lazy_static::lazy_static! {
    pub static ref URL: &'static str = "redis://127.0.0.1";
    static ref PORT: u16 = 6379;
    pub static ref POOL: r2d2::Pool<redis::Client> = r2d2::Pool::builder()
        .build(redis::Client::open(format!("{}:{}", *URL, *PORT))
               .unwrap())
        .unwrap();
}

#[cfg(not(any(test, debug_assertions)))]
lazy_static::lazy_static! {
    static ref URL: &'static str = "redis://redis";
    static ref PORT: u16 = 6379;
    pub static ref POOL: r2d2::Pool<redis::Client> = r2d2::Pool::builder()
        .build(redis::Client::open(format!("{}:{}", *URL, *PORT))
               .unwrap())
        .unwrap();
}
