extern crate thiserror;

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("No bid/asks to match order against")]
    EmptyDepth,
    #[error("Market Depth falls short of {0} quantity to satisfy order.")]
    InsufficientDepth(i32),
}
