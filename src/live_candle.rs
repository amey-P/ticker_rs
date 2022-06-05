use chrono::{DateTime, Duration, Utc};
use redis;

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
    pub volume: u64,
    pub timestamp: DateTime<Utc>,
    pub period: Duration,
}

impl Candle {
    pub fn new(period: Duration) -> Self {
        Candle {
            period,
            ..Default::default()
        }
    }

    fn update(&mut self, key: String, value: &redis::Value) {
        match key.as_str() {
            "open" => self.open = redis::from_redis_value(value).unwrap(),
            "high" => self.high = redis::from_redis_value(value).unwrap(),
            "low" => self.low = redis::from_redis_value(value).unwrap(),
            "close" => self.close = redis::from_redis_value(value).unwrap(),
            "volume" => self.volume = {
                let value: f64 = redis::from_redis_value(value).unwrap();
                value as u64
            },
            k => panic!("Unrecognized key '{}' found in OHLC database!", k),
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
            volume: 0,
            timestamp: Utc::now(),
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
}
