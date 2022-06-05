use crate::tickers::Ticker;
use chrono::{Utc, DateTime, NaiveDateTime, TimeZone};
use crate::utils::POOL;
use crate::live_candle::{Candle, DATETIME_FMT, EXPIRE_SEC};

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

    fn candle_ts(&self) -> Vec<DateTime<Utc>> {
        let mut connection = POOL.clone().get().unwrap();
        let all_candles_wildcard = format!("{}:CANDLES:*", self.key());
        let cmd = redis::Cmd::keys(all_candles_wildcard);
        let all_keys: Vec<String> = cmd.query(&mut *connection).unwrap();

        let mut timestamps: Vec<DateTime<Utc>> = all_keys
            .into_iter()
            .map(|x| x.split(':').last().unwrap().to_string())
            .map(|x| {
                let dt = NaiveDateTime::parse_from_str(x.as_str(), &DATETIME_FMT).unwrap();
                Utc::from_utc_datetime(&Utc, &dt)
            })
            .collect();
        timestamps.sort();

        timestamps
    }

    fn latest_candle(&self) -> Option<Candle> {
        match self.candle_ts().last() {
            Some(ts) => self.candle_from_timestamp(*ts),
            None => None,
        }
    }

    fn candle_from_timestamp(&self, timestamp: DateTime<Utc>) -> Option<Candle> {
        let mut connection = POOL.clone().get().unwrap();

        let ts_string = timestamp.format(DATETIME_FMT.as_str());
        let query_key = format!("{}:CANDLES:{}", self.key(), ts_string);
        
        if redis::Cmd::exists(query_key.clone()).query(&mut *connection).unwrap() {
            let mut candle: Candle = redis::Cmd::hgetall(query_key)
                                        .query(&mut *connection)
                                        .unwrap();
            candle.timestamp = timestamp;
            Some(candle)
        }
        else {
            None
        }
    }

    fn update_candle(&self, candle: Candle) {
        let mut connection = POOL.clone().get().unwrap();

        let timestamp = candle.timestamp.format(&DATETIME_FMT);
        let query_key = format!("{}:CANDLES:{}", self.key(), timestamp);

        redis::Cmd::hset_multiple(query_key.clone(),
                                  &[("open", candle.open),
                                    ("high", candle.high),
                                    ("low", candle.low),
                                    ("close", candle.close),
                                    ("volume", candle.volume as f64)]).execute(&mut *connection);

        redis::Cmd::expire(query_key, *EXPIRE_SEC).execute(&mut *connection);
    }
}

