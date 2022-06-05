use crate::{redis_utils::RedisScrip, scrip::Scrip, utils::POOL};
use crate::scrip::{Exchange, ExchangeType};
use chrono::prelude::*;
use std::collections::HashMap;

lazy_static::lazy_static! {
    pub static ref EXPIRY_FORMAT: String = String::from("%d/%m/%Y");
}

#[derive(Debug, Clone)]
pub enum OptionType {
    CE,
    PE,
}

impl std::fmt::Display for OptionType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// =============================================================================
//                      Single Option Ticker @ Strike, CE/PE
// =============================================================================

#[derive(Clone, Debug)]
pub struct OptionScrip {
    pub name: String,
    pub exchange: Exchange,
    pub exchange_type: ExchangeType,
    pub strike: u32,
    pub option_type: OptionType,
    pub expiry: NaiveDate,
    pub underlying: Option<Box<Scrip>>,
}

impl RedisScrip for OptionScrip {
    fn key(&self) -> String {
        let expiry = self.expiry.format(&*EXPIRY_FORMAT);
        format!(
            "{}:{}:{}:{}:{}:{}",
            self.name, self.exchange.to_string(), self.exchange_type.to_string(),
            expiry, self.strike, self.option_type
        )
    }
}

impl OptionScrip {
    pub fn new(
        name: &str,
        exchange: &str,
        exchange_type: &str,
        expiry: NaiveDate,
        strike: u32,
        option_type: OptionType,
        underlying: Option<Scrip>,
    ) -> Self {
        let boxed_underlying = underlying.map(Box::new);
        Self {
            name: name.to_string(),
            exchange: exchange.into(),
            exchange_type: exchange_type.into(),
            expiry,
            strike,
            option_type,
            underlying: boxed_underlying,
        }
    }
}
// =============================================================================
//                                Option Chain
// =============================================================================

#[derive(Clone, Debug)]
pub struct OptionChainScrip {
    pub name: String,
    pub exchange: Exchange,
    pub exchange_type: ExchangeType,
    pub expiry: NaiveDate,
    pub underlying: Option<Scrip>,
}

impl OptionChainScrip {
    pub fn new(name: &str, exchange: &str, exchange_type: &str, expiry: NaiveDate, underlying: Option<Scrip>) -> Self {
        Self {
            name: name.to_string(),
            exchange: exchange.into(),
            exchange_type: exchange_type.into(),
            expiry,
            underlying,
        }
    }

    fn key(&self) -> String {
        format!("{}:{}:{}*", self.name, self.exchange.to_string(), self.exchange_type.to_string())
    }

    fn sub_keys(&self) -> Vec<String> {
        let mut command = redis::Cmd::new();
        let key = self.key();

        command.arg("KEYS").arg(key);
        let mut connection = POOL.clone().get().unwrap();
        let keys: Vec<String> = command.query(&mut *connection).unwrap();
        keys
    }
}

// =============================================================================

#[derive(Debug, Clone)]
pub struct OptionChain {
    pub scrip: OptionChainScrip,
    pub calls: HashMap<u32, OptionScrip>,
    pub puts: HashMap<u32, OptionScrip>,
}

impl OptionChain {
    pub fn new(
        name: &str,
        exchange: &str,
        exchange_type: &str,
        expiry: NaiveDate,
        underlying: Option<Scrip>
    ) -> Self {
        let mut option_chain_ticker = Self {
            scrip: OptionChainScrip::new(name, exchange, exchange_type, expiry, underlying),
            calls: HashMap::new(),
            puts: HashMap::new(),
        }; 
        option_chain_ticker.refresh_chain();

        option_chain_ticker
    }

    pub fn strikes(&self) -> Vec<u32> {
        let mut strikes: Vec<u32> = self.calls.keys().copied().collect();
        strikes.sort_unstable();
        strikes
    }

    pub fn at_strike(&self, strike: &u32) -> (OptionScrip, OptionScrip) {
        let call = self.calls.get(strike).unwrap();
        let put = self.puts.get(strike).unwrap();
        (call.clone(), put.clone())
    }

    pub fn filter_strikes_with(&mut self, filter: impl Fn(&Self, u32) -> bool) -> &mut Self {
        for strike in self.strikes().iter() {
            if filter(self, *strike) {
                continue;
            }

            self.calls.remove(strike);
            self.puts.remove(strike);
        }

        self
    }

    // pub fn reload(&mut self) {
    //     self.calls.values_mut().for_each(|c| {
    //         c.reload();
    //     });
    //     self.puts.values_mut().for_each(|p| {
    //         p.reload();
    //     });
    // }

    // Update the Strikes in `calls` and `puts` according to Redis
    // Adds appropriate `OptionTickers` in calls and puts for strikeprices
    // that were not present earlier
    pub fn refresh_chain(&mut self) -> &mut Self {
        let keys: Vec<String> = self.scrip.sub_keys();

        for strike in keys.into_iter().map(|k| {
            k.split(':').collect::<Vec<&str>>()[3]
                .parse::<u32>()
                .unwrap()
        }) {
            match self.calls.get_mut(&strike) {
                Some(_) => (),
                None => {
                    let opt_scrip = OptionScrip {
                        name: self.scrip.name.clone(),
                        exchange: self.scrip.exchange,
                        exchange_type: self.scrip.exchange_type,
                        expiry : self.scrip.expiry,
                        strike,
                        option_type: OptionType::PE,
                        underlying: self.scrip.underlying.clone().map(Box::new),
                    };
                    
                    self.calls.insert(
                        strike,
                        opt_scrip,
                    );
                }
            };
            match self.puts.get_mut(&strike) {
                Some(_) => (),
                None => {
                    let opt_scrip = OptionScrip {
                        name: self.scrip.name.clone(),
                        exchange: self.scrip.exchange,
                        exchange_type: self.scrip.exchange_type,
                        expiry : self.scrip.expiry,
                        strike,
                        option_type: OptionType::PE,
                        underlying: self.scrip.underlying.clone().map(Box::new),
                    };
                    
                    self.puts.insert(
                        strike,
                        opt_scrip,
                    );
                }
            };
        }
        self
    }

    pub fn sanity_check(&self) -> bool {
        // TODO
        true
    }
}
