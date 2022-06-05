use aws_sdk_dynamodb::model::AttributeValue;
use aws_types::sdk_config::SdkConfig;
use aws_sdk_dynamodb::Client;
use crate::tickers::Ticker;
use crate::options::EXPIRY_FORMAT;
use crate::{IndexScrip, StockScrip, OptionScrip, OptionType};
use crate::redis_utils::RedisScrip;
use crate::info::{MetaData, TABLE_NAME};
use chrono::NaiveDate;
use redis;
use std::hash::{Hash, Hasher};
use cached::proc_macro::cached;

#[derive(Copy, Debug, Clone)]
pub enum Exchange {
    NSE,
    BSE,
    MCX,
}

impl From<&'_ str> for Exchange {
    fn from(exchange_key: &str) -> Self {
        match exchange_key {
            "NSE" => Exchange::NSE,
            "BSE" => Exchange::BSE,
            "MCX" => Exchange::MCX,
            exch => panic!("Invalid key. Exchange {} not mapped", exch),
        }
    }
}

impl ToString for Exchange {
    fn to_string(&self) -> String {
        match &self {
            Exchange::NSE => String::from("NSE"),
            Exchange::BSE => String::from("BSE"),
            Exchange::MCX => String::from("MCX"),
        }
    }
}

#[derive(Copy, Debug, Clone)]
pub enum ExchangeType {
    Cash,
    Index,
    // Instead of using "Derivative" as is the case in real markets, segregating
    // Options and Futures might be useful for parsing the key which is the true
    // purpose of the enum.
    Options,
    Futures,
}
impl From<&'_ str> for ExchangeType {
    fn from(exchange_type_key: &str) -> Self {
        match exchange_type_key {
            "C" => ExchangeType::Cash,
            "I" => ExchangeType::Index,
            "O" => ExchangeType::Options,
            "F" => ExchangeType::Futures,
            exch => panic!("Invalid key. Exchange type {} not mapped", exch),
        }
    }
}

impl ToString for ExchangeType {
    fn to_string(&self) -> String {
        match &self {
            ExchangeType::Cash => String::from("C"),
            ExchangeType::Index => String::from("I"),
            ExchangeType::Options => String::from("O"),
            ExchangeType::Futures => String::from("F"),
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

#[cached]
fn dynamo_call(name: String) -> Option<MetaData> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build().unwrap();

    let shared_config: SdkConfig = runtime.block_on(aws_config::load_from_env());
    let client = Client::new(&shared_config);

    let request = client
        .get_item()
        .table_name(TABLE_NAME)
        .key(
            "scrip",
            AttributeValue::S(name),
        );

    let response = runtime.block_on(request.send());
    match response {
        Ok(r) => {
            Some(MetaData::from_response(r.item.as_ref().unwrap()))
        },
        Err(_) => {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub enum Scrip {
    Stock(StockScrip),
    Index(IndexScrip),
    Option(OptionScrip),
}

impl Scrip {
    pub fn from_key(key: &str) -> Self {
        let parts: Vec<&str> = key.split(':').collect();

        let name = parts[0];
        let exchange: Exchange = parts[1].into();
        let exchange_type: ExchangeType = parts[2].into();

        if parts.len() == 3 {
            return match exchange_type {
                ExchangeType::Cash => Scrip::Stock(StockScrip { name: name.to_string(), exchange, exchange_type }),
                ExchangeType::Index => Scrip::Index(IndexScrip { name: name.to_string(), exchange, exchange_type }),
                _ => panic!("Invalid key -> {}", key),
            }
        }

        // Handling Futures and Options
        match exchange_type {
            ExchangeType::Options => {
                let expiry: NaiveDate = NaiveDate::parse_from_str(parts[3], &*EXPIRY_FORMAT).unwrap();
                let strike: u32 = parts[4].parse::<u32>().unwrap();
                let option_type = match parts[5] {
                    "CE" => OptionType::CE,
                    "PE" => OptionType::PE,
                    _ => panic!("Assuming 6th position for CE/PE. Instead found {}", parts[5]),
                };
                let underlying = Some(Box::new(Scrip::from_key(&parts[..4].join(":"))));
                Scrip::Option(OptionScrip {
                    name: name.to_string(),
                    exchange,
                    exchange_type,
                    strike,
                    option_type,
                    expiry,
                    underlying,
                })
            },
            ExchangeType::Futures => {
                todo!()
            },
            _ => panic!("Invalid key -> {}", key),
        }
    }

    pub fn get_metadata(&self) -> Option<MetaData> {
        let name = match self {
            Scrip::Stock(stock) => stock.name.clone(),
            Scrip::Index(index) => index.name.clone(),
            Scrip::Option(option) => option.name.clone(),
        };

        dynamo_call(name)
    }
}

impl RedisScrip for Scrip {
    fn key(&self) -> String {
        match self {
            Scrip::Stock(s) => s.key(),
            Scrip::Index(i) => i.key(),
            Scrip::Option(o) => o.key(),
        }
    }

    fn updated_ticker(&self) -> Ticker {
        match self {
            Scrip::Stock(s) => s.updated_ticker(),
            Scrip::Index(i) => i.updated_ticker(),
            Scrip::Option(o) => o.updated_ticker(),
        }
    }

    fn ticker_command(&self) -> redis::Cmd {
        match self {
            Scrip::Stock(s) => s.ticker_command(),
            Scrip::Index(i) => i.ticker_command(),
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
