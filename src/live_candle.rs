use chrono::prelude::*;


pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub timestamp: DateTime,
    pub period: Duration,
}

pub struct CandleChart {
    pub timeseries: HashMap<DateTime, Candle>,
}
