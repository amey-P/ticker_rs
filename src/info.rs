use aws_sdk_dynamodb::model::AttributeValue;
use std::collections::HashMap;
use chrono::prelude::*;

pub static TABLE_NAME: &str = "scrip_info";

pub fn format_timezone(timezone: &str) -> FixedOffset {
    let direction = timezone.chars().next().unwrap();
    let hours = timezone[1..3].parse::<i32>().unwrap();
    let minutes = timezone[4..6].parse::<i32>().unwrap();

    let seconds = (hours*60 + minutes) * 60;
    match direction {
        '-' => FixedOffset::west(seconds),
        '+' => FixedOffset::east(seconds),
        _ => panic!("Invalid timezone format => {}", timezone),
    }
}

pub fn format_time(time: &str) -> String {
    format!("{}.{}", &time[0..2], &time[2..4])
}

#[derive(Clone, Debug)]
pub struct IndexMetaData {
    pub scrip: String,
    pub open_time: String,
    pub close_time: String,
    pub currency: String,
    pub exchange: String,
    pub timezone: FixedOffset,
    pub constituents: HashMap<String, f64>,
}

impl IndexMetaData {
    pub fn from_response(items: &HashMap<String, AttributeValue>) -> Self {
        let mut index_meta_data = IndexMetaData {
            scrip: String::new(),
            open_time: String::new(),
            close_time: String::new(),
            currency: String::new(),
            exchange: String::new(),
            timezone: FixedOffset::east(0),
            constituents: HashMap::new(),
        };
        items.iter().for_each(|(k, v)| index_meta_data.update(k, v));
        index_meta_data
    }
    pub fn update(&mut self, key: &str, attribute: &AttributeValue) {
        match key {
            "scrip" => if let AttributeValue::S(scrip) = attribute {
                self.scrip = scrip.to_string();
            }
            "openTime" => if let AttributeValue::S(open_time) = attribute {
                self.open_time = format_time(open_time);
            }
            "closeTime" => if let AttributeValue::S(close_time) = attribute {
                self.close_time = format_time(close_time);
            }
            "currency" => if let AttributeValue::S(currency) = attribute {
                self.currency = currency.to_uppercase();
            }
            "exchange" => if let AttributeValue::S(exchange) = attribute {
                self.exchange = exchange.to_uppercase();
            }
            "timezone" => if let AttributeValue::S(timezone) = attribute {
                self.timezone = format_timezone(timezone);
            }
            "constituents" => if let AttributeValue::M(constituents) = attribute {
                self.constituents = HashMap::new();
                constituents
                    .iter()
                    .for_each(|(scrip, information)| {
                        if let AttributeValue::N(weight) = information {
                            let weight: f64 = weight.parse::<f64>().unwrap();
                            self.constituents.insert(scrip.to_string(), weight);
                        }
                    });
            }
            _ => (),
        }
        
    }
}

// =============================================================================
//                                 STOCK METADATA
// =============================================================================

#[derive(Clone, Debug)]
pub struct StockMetaData {
    pub scrip: String,
    pub open_time: String,
    pub close_time: String,
    pub currency: String,
    pub exchange: String,
    pub timezone: FixedOffset,
    pub free_float_market_cap: f64,
}

impl StockMetaData {
    pub fn from_response(items: &HashMap<String, AttributeValue>) -> Self {
        let mut stock_meta_data = StockMetaData {
            scrip: String::new(),
            open_time: String::new(),
            close_time: String::new(),
            currency: String::new(),
            exchange: String::new(),
            timezone: FixedOffset::east(0),
            free_float_market_cap: 0.0,
        };
        items.iter().for_each(|(k, v)| stock_meta_data.update(k, v));
        stock_meta_data
    }
    pub fn update(&mut self, key: &str, attribute: &AttributeValue) {
        match key {
            "scrip" => if let AttributeValue::S(scrip) = attribute {
                self.scrip = scrip.to_string();
            }
            "openTime" => if let AttributeValue::S(open_time) = attribute {
                self.open_time = format_time(open_time);
            }
            "closeTime" => if let AttributeValue::S(close_time) = attribute {
                self.close_time = format_time(close_time);
            }
            "currency" => if let AttributeValue::S(currency) = attribute {
                self.currency = currency.to_uppercase();
            }
            "exchange" => if let AttributeValue::S(exchange) = attribute {
                self.exchange = exchange.to_uppercase();
            }
            "freeFloatMarketCap" => if let AttributeValue::N(ff_mcap) = attribute {
                self.free_float_market_cap = ff_mcap.parse::<f64>().unwrap();
            }
            "timezone" => if let AttributeValue::S(timezone) = attribute {
                self.timezone = format_timezone(timezone);
            }
            _ => (),
        }
    }
}

// =============================================================================
//                                 GENRAL METADATA
// =============================================================================

#[derive(Clone, Debug)]
pub enum MetaData {
    Index(IndexMetaData),
    Stock(StockMetaData),
}

impl MetaData {
    pub fn from_response(items: &HashMap<String, AttributeValue>) -> Self {
        match items.get("type") {
            Some(t) => {
                if let AttributeValue::S(ty) = t {
                    return match ty.as_str() {
                        "cash" => MetaData::Stock(StockMetaData::from_response(items)),
                        "index" => MetaData::Index(IndexMetaData::from_response(items)),
                        attr => panic!("Invalid `type` key -> {}", attr),
                    }
                }
                panic!("`type` key not in String format.")
            }
            None => panic!("Key `type` missing. Unable to identify scrip type!"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::IndexScrip;
    use crate::scrip::Scrip;
    use crate::stock::StockScrip;

    // =========================================================================
    // These tests passes only if the program has access to the dynamodb table.
    #[test]
    pub fn sbin_call() {
        let sbin = Scrip::Stock(StockScrip::new("SBIN", "NSE", "C"));
        let sbin_info = sbin.get_metadata();
        match sbin_info {
            Some(info) => {
                dbg!(info);
            },
            None => panic!("Empty SBI"),
        }
    }
    #[test]
    pub fn nifty50_call() {
        let nifty = Scrip::Index(IndexScrip::new("NIFTY", "NSE", "I"));
        let nifty_info = nifty.get_metadata();
        match nifty_info {
            Some(info) => {
                dbg!(info);
            },
            None => panic!("Empty Nifty"),
        }
    }
    // =========================================================================
}
