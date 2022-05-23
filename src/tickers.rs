use serde::{Serialize, Deserialize};
use crate::scrip::{RedisScrip, Scrip};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct OHLC {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct DepthOrder {
    pub price: f64,
    pub quantity: u32,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Depth {
    pub depth: u8,
    pub total_bid: u32,
    pub total_ask: u32,
    pub bid: Vec<DepthOrder>,
    pub ask: Vec<DepthOrder>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub ltp: f64,
    pub ohlc: OHLC,
    pub depth: Depth,
}

impl Ticker {
    pub fn new() -> Self {
        let ticker: Ticker = Default::default();
        ticker
    }

    #[cfg(test)]
    pub fn reload(&mut self, command: &redis::Cmd) -> &mut Self {
        use crate::test_util::TEST_TICKER_1;
        self.update_from_ticker(TEST_TICKER_1.clone());
        self
    }

    #[cfg(not(test))]
    pub fn reload(&mut self, command: &redis::Cmd) -> &mut Self {
        let mut connection = crate::utils::POOL.clone().get().unwrap();
        let new_values: Ticker = command.query(&mut *connection).unwrap();
        self.update_from_ticker(new_values);
        self
    }

    pub fn update_from_ticker(&mut self, updated_ticker: Ticker) {
        self.ltp = updated_ticker.ltp;
        self.ohlc = updated_ticker.ohlc;
        self.depth = updated_ticker.depth;
    }

    fn update_depth(&mut self, key: String, value: &redis::Value) {
        // For Depth -> Bid/Ask List
        let parts: Vec<&str> = key.split(":").collect();
        if parts.len() != 3 {
            panic!("Un-mapped key -> {}", key);
        }

        if let [bid_or_ask, rate_or_qty, val] = &parts[..3] {
            let idx = val.parse::<usize>().unwrap();
            let target: &mut Vec<DepthOrder>;
            match bid_or_ask {
                &"bid" => target = &mut self.depth.bid,
                &"ask" => target = &mut self.depth.ask,
                _ => panic!("Un-mapped key -> {}", key),
            }
            match target.get_mut(idx) {
                Some(order) => match rate_or_qty {
                    &"rate" => order.price = redis::from_redis_value(value).unwrap(),
                    &"quantity" => order.quantity = redis::from_redis_value(value).unwrap(),
                    _ => panic!("Un-mapped key -> {}", key),
                },
                None => {
                    target.resize(idx + 1, Default::default());
                    match rate_or_qty {
                        &"rate" => {
                            target[idx].price = redis::from_redis_value(value).unwrap()
                        }
                        &"quantity" => {
                            target[idx].quantity = redis::from_redis_value(value).unwrap()
                        }
                        _ => panic!("Un-mapped key -> {}", key),
                    }
                }
            }
        }
    }

    // Expects a strict template of fields.
    pub fn update(&mut self, key: String, value: &redis::Value) {
        match key.as_str() {
            "ltp" => self.ltp = redis::from_redis_value(value).unwrap(),
            "open" => self.ohlc.open = redis::from_redis_value(value).unwrap(),
            "high" => self.ohlc.high = redis::from_redis_value(value).unwrap(),
            "low" => self.ohlc.low = redis::from_redis_value(value).unwrap(),
            "close" => self.ohlc.close = redis::from_redis_value(value).unwrap(),
            "depth" => self.depth.depth = redis::from_redis_value(value).unwrap(),
            "total_bid" => self.depth.total_bid = redis::from_redis_value(value).unwrap(),
            "total_ask" => self.depth.total_ask = redis::from_redis_value(value).unwrap(),
            "total_volume" => self.ohlc.volume = redis::from_redis_value(value).unwrap(),
            depth_key => self.update_depth(depth_key.to_string(), value),
        }
    }
}

impl redis::FromRedisValue for Ticker {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        let mut ticker: Ticker = Default::default();
        v.as_map_iter()
            .ok_or_else(|| panic!("Empty values found on Redis server"))
            .unwrap_or_else(|_| panic!("Empty values found on Redis server"))
            .for_each(|(k, v)| ticker.update(redis::from_redis_value(k).unwrap(), v));
        Ok(ticker)
    }
}

#[derive(Clone)]
pub struct CompleteTicker {
    pub ticker: Ticker,
    pub scrip: Scrip,
}

impl CompleteTicker {
    pub fn from_scrip(scrip: Scrip) -> Self {
        let ticker = scrip.updated_ticker();
        Self { ticker, scrip }
    }

    pub fn reload(&mut self) {
        let updated_ticker = self.scrip.updated_ticker();
        self.ticker.update_from_ticker(updated_ticker);
    }
}

impl std::fmt::Debug for CompleteTicker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompleteTicker")
            .field("scrip", &self.scrip.key())
            .field("ticker", &self.ticker)
            .finish()
    }
}
