use lnpbp::bitcoin::hashes::hex;
use std::num::{ParseFloatError, ParseIntError};

#[derive(Clone, Copy, Debug, Display, Error)]
#[display_from(Debug)]
pub struct ParseError;

impl From<ParseFloatError> for ParseError {
    fn from(err: ParseFloatError) -> Self {
        Self
    }
}

impl From<ParseIntError> for ParseError {
    fn from(err: ParseIntError) -> Self {
        Self
    }
}

impl From<hex::Error> for ParseError {
    fn from(err: hex::Error) -> Self {
        Self
    }
}

/// Error used to communicate across FFI & WASM calls
#[derive(Clone, Debug, Display)]
#[display_from(Debug)]
pub struct InteroperableError(pub String);

impl<T> From<T> for InteroperableError
where
    T: std::error::Error,
{
    fn from(err: T) -> Self {
        Self(format!("{}", err))
    }
}
