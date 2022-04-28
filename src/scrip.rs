use crate::{options::OptionScrip, utils::POOL};
use crate::stock::StockScrip;
use crate::tickers::Ticker;
use crate::live_candle::{Candle, DATETIME_FMT};
use chrono::{Local, DateTime, NaiveDateTime, TimeZone};
use redis;
use std::hash::{Hash, Hasher};

pub trait RedisScrip {
    fn key(&self) -> String;

    fn ticker_command(&self) -> redis::Cmd {
        let key = self.key();
        let mut cmd = redis::Cmd::new();

        cmd.arg("HGETALL").arg(key);
        cmd
    }

    fn updated_ticker(&self) -> Ticker {
        let mut ticker = Ticker::new();
        ticker.reload(&self.ticker_command());
        ticker
    }

    fn candle_ts(&self) -> Vec<DateTime<Local>> {
        let mut connection = POOL.clone().get().unwrap();
        let all_candles_wildcard = format!("{}:CANDLES:*", self.key());
        let cmd = redis::Cmd::keys(all_candles_wildcard);
        let all_keys: Vec<String> = cmd.query(&mut *connection).unwrap();

        let mut timestamps: Vec<DateTime<Local>> = all_keys
            .into_iter()
            .map(|x| x.split(":").last().unwrap().to_string())
            .map(|x| {
                let dt = NaiveDateTime::parse_from_str(x.as_str(), &DATETIME_FMT).unwrap();
                Local.from_local_datetime(&dt).unwrap()
            })
            .collect();
        timestamps.sort();

        timestamps
    }

    fn latest_candle(&self) -> Option<Candle> {
        match self.candle_ts().last() {
            Some(ts) => Candle::from_timestamp(self, ts.clone()),
            None => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RawScrip {
    pub key: String,
}

impl RedisScrip for RawScrip {
    fn key(&self) -> String {
        self.key.clone()
    }
}

#[derive(Clone, Debug)]
pub enum Scrip {
    Stock(StockScrip),
    Option(OptionScrip),
}

impl RedisScrip for Scrip {
    fn key(&self) -> String {
        match self {
            Scrip::Stock(s) => s.key(),
            Scrip::Option(o) => o.key(),
        }
    }

    fn updated_ticker(&self) -> Ticker {
        match self {
            Scrip::Stock(s) => s.updated_ticker(),
            Scrip::Option(o) => o.updated_ticker(),
        }
    }

    fn ticker_command(&self) -> redis::Cmd {
        match self {
            Scrip::Stock(s) => s.ticker_command(),
            Scrip::Option(o) => o.ticker_command(),
        }
    }
}

impl Hash for Scrip {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key().hash(state);
    }
}

impl PartialEq for Scrip {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl Eq for Scrip {}
