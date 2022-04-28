use crate::scrip::RedisScrip;

#[derive(Clone, Debug)]
pub struct StockScrip {
    pub name: String,
    pub exchange: String,
    pub exchange_type: String,
}

impl StockScrip {
    pub fn new(name: &str, exchange: &str, exchange_type: &str) -> Self {
        Self {
            name: name.to_string(),
            exchange: exchange.to_string(),
            exchange_type: exchange_type.to_string(),
        }
    }
}

impl RedisScrip for StockScrip {
    fn key(&self) -> String {
        format!("{}:{}", self.name, self.exchange)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candle_timestamps() {
        let sbin: StockScrip = StockScrip::new("SBIN", "N", "C");
        assert!(sbin.candle_ts().len() > 0);
    }

    #[test]
    fn latest_candle() {
        let sbin: StockScrip = StockScrip::new("SBIN", "N", "C");
        let candle = sbin.latest_candle().unwrap();
        assert!(candle.open != 0.0);
        assert!(candle.high != 0.0);
        assert!(candle.low != 0.0);
        assert!(candle.close != 0.0);
    }
}
