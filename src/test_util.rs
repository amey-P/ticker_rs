use lazy_static::lazy_static;
use crate::tickers::*;

lazy_static! {
    pub static ref TEST_TICKER_1: Ticker = Ticker {
        ltp: 400.23,
        ohlc: OHLC {
            open: 392.23,
            high: 420.69,
            low: 390.44,
            close: 400.23,
            volume: 1234234,
        },
        depth: Depth {
            depth: 5,
            total_bid: 30000,
            total_ask: 50000,
            bid: vec![
                DepthOrder {
                    price: 400.15,
                    quantity: 5,
                },
                DepthOrder {
                    price: 400.13,
                    quantity: 4,
                },
                DepthOrder {
                    price: 400.0,
                    quantity: 2,
                },
                DepthOrder {
                    price: 399.15,
                    quantity: 2,
                },
                DepthOrder {
                    price: 398.15,
                    quantity: 1,
                },
            ],
            ask: vec![
                DepthOrder {
                    price: 401.15,
                    quantity: 5,
                },
                DepthOrder {
                    price: 402.13,
                    quantity: 4,
                },
                DepthOrder {
                    price: 403.0,
                    quantity: 2,
                },
                DepthOrder {
                    price: 403.15,
                    quantity: 2,
                },
                DepthOrder {
                    price: 404.15,
                    quantity: 1,
                },
            ],
        },
    };
}
