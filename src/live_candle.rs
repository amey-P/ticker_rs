use chrono::{DateTime, Duration, Local};
use redis;

use crate::utils::POOL;
use crate::scrip::RedisScrip;

lazy_static::lazy_static! {
    pub static ref DATETIME_FMT: String = String::from("%Y/%m/%d-%H.%M");
    pub static ref EXPIRE_SEC: usize = 7*24*60*60;
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
        let query_key = format!("{}:CANDLES:{}", scrip.key(), ts_string);
        
        if redis::Cmd::exists(query_key.clone()).query(&mut *connection).unwrap() {
            let mut candle: Candle = redis::Cmd::hgetall(query_key)
                                        .query(&mut *connection)
                                        .unwrap();
            candle.timestamp = timestamp;
            return Some(candle);
        }
        else {
            return None;
        }
    }

    pub fn push_redis(&self, scrip: &(impl RedisScrip + ?Sized)) {
        let mut connection = POOL.clone().get().unwrap();

        let timestamp = self.timestamp.format(&DATETIME_FMT);
        let query_key = format!("{}:CANDLES:{}", scrip.key(), timestamp);

        redis::Cmd::hset_multiple(query_key.clone(),
                                  &[("open", self.open),
                                    ("high", self.high),
                                    ("low", self.low),
                                    ("close", self.close)]).execute(&mut *connection);

        redis::Cmd::expire(query_key, *EXPIRE_SEC).execute(&mut *connection);
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

#[cfg(test)]
mod test {
    use crate::RawScrip;
    use super::*;

    #[test]
    fn upload_and_fetch_candle() {
        let mut candle: Candle = Default::default();
        candle.timestamp = Local::now();

        let test_scrip = RawScrip { key: "TEST".to_string() };
        candle.push_redis(&test_scrip);
        let uploaded_candle = Candle::from_timestamp(&test_scrip, candle.timestamp).unwrap();
        let original_minute = format!("{:?}", uploaded_candle.timestamp.format("%Y/%m/%d-%H:%M"));
        let uploaded_minute = format!("{:?}", candle.timestamp.format("%Y/%m/%d-%H:%M"));
        assert_eq!(original_minute, uploaded_minute);

        // Cleanup
        let mut connection = POOL.clone().get().unwrap();
        let del_key = format!("TEST:CANDLES:{}", candle.timestamp.format(&DATETIME_FMT));
        redis::Cmd::del(del_key).execute(&mut *connection);
    }
}
