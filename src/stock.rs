use crate::scrip::RedisScrip;
use crate::tickers::Ticker;

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

    fn updated_ticker(&self) -> Ticker {
        let mut ticker = Ticker::new();
        ticker.reload(&self.get_command());
        ticker
    }
}
