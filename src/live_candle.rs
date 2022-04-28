use chrono::{DateTime, Duration, Local};
use redis;

use crate::utils::POOL;
use crate::scrip::RedisScrip;

lazy_static::lazy_static! {
    pub static ref DATETIME_FMT: String = String::from("%Y/%m/%d-%H.%M");
}

#[derive(Clone, Debug)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub timestamp: DateTime<Local>,
    pub period: Duration,
}

impl Candle {
    pub fn new(period: Duration) -> Self {
        let mut candle: Candle = Default::default();
        candle.period = period;

        candle
    }

    pub fn from_timestamp(scrip: &(impl RedisScrip + ?Sized), timestamp: DateTime<Local>) -> Option<Self> {
        let mut connection = POOL.clone().get().unwrap();

        let ts_string = timestamp.format(DATETIME_FMT.as_str());
        let query_key = format!("{}:{}", scrip.key(), ts_string);
        
        if let Some(mut candle) = redis::Cmd::hgetall(query_key).query::<Candle>(&mut *connection).ok() {
            candle.timestamp = timestamp;
            return Some(candle);
    }
        else {
            return None;
        }
    }

    fn push(&self, key: String) {
        let connection = POOL.clone().get().unwrap();

        let timestamp = self.timestamp.format("%Y/%m/%d-%H.%m");
        let query_key = format!("{}:{}", key, timestamp);
        let cmd = redis::Cmd::new();

        todo!();
    }

    fn update(&mut self, key: String, value: &redis::Value) {
        match key.as_str() {
            "open" => self.open = redis::from_redis_value(value).unwrap(),
            "high" => self.high = redis::from_redis_value(value).unwrap(),
            "low" => self.low = redis::from_redis_value(value).unwrap(),
            "close" => self.close = redis::from_redis_value(value).unwrap(),
            _ => (),
        }
    }
}

impl Default for Candle {
    fn default() -> Self {
        Candle {
            open: 0.0,
            high: 0.0,
            low: 0.0,
            close: 0.0,
            timestamp: Local::now(),
            period: Duration::minutes(1),
        }
    }
}

impl redis::FromRedisValue for Candle {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        let mut candle: Candle = Default::default();
        v.as_map_iter()
            .ok_or_else(|| panic!("Empty values found on Redis server"))
            .unwrap_or_else(|_| panic!("Empty values found on Redis server"))
            .for_each(|(k, v)| candle.update(redis::from_redis_value(k).unwrap(), v));
        Ok(candle)
    }
}
