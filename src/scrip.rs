use crate::options::OptionScrip;
use crate::stock::StockScrip;
use crate::tickers::Ticker;
use redis;
use std::hash::{Hash, Hasher};

pub trait RedisScrip {
    fn key(&self) -> String;

    fn get_command(&self) -> redis::Cmd {
        let key = self.key();
        let mut cmd = redis::Cmd::new();

        cmd.arg("HGETALL").arg(key);
        cmd
    }

    fn updated_ticker(&self) -> Ticker {
        let mut ticker = Ticker::new();
        ticker.reload(&self.get_command());
        ticker
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

    fn get_command(&self) -> redis::Cmd {
        match self {
            Scrip::Stock(s) => s.get_command(),
            Scrip::Option(o) => o.get_command(),
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
