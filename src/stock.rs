use crate::redis_utils::RedisScrip;
use crate::scrip::{Exchange, ExchangeType};


#[derive(Clone, Debug)]
pub struct StockScrip {
    pub name: String,
    pub exchange: Exchange,
    pub exchange_type: ExchangeType,
}

impl StockScrip {
    pub fn new(name: &str, exchange: &str, exchange_type: &str) -> Self {
        Self {
            name: name.to_string(),
            exchange: exchange.into(),
            exchange_type: exchange_type.into(),
        }
    }
}

impl RedisScrip for StockScrip {
    fn key(&self) -> String {
        format!("{}:{}:{}", 
                self.name, 
                self.exchange.to_string(), 
                self.exchange_type.to_string())
    }
}

pub type IndexScrip = StockScrip;

#[cfg(test)]
mod tests {
}
