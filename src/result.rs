use std::{error, fmt, io, num, result};

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    CSV(csv::Error),
    BinanceClient(binance_client::errors::Error),
    Diesel(diesel::result::Error),
    ParseStr(String),
    TryFromNumber(num::TryFromIntError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IO(error) => fmt::Display::fmt(error, f),
            Self::CSV(error) => fmt::Display::fmt(error, f),
            Self::BinanceClient(error) => fmt::Display::fmt(error, f),
            Self::Diesel(error) => fmt::Display::fmt(error, f),
            Self::ParseStr(source) => fmt::Display::fmt(source, f),
            Self::TryFromNumber(error) => fmt::Display::fmt(error, f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::IO(error) => Some(error),
            Self::CSV(error) => Some(error),
            Self::BinanceClient(error) => Some(error),
            Self::Diesel(error) => Some(error),
            Self::ParseStr(_) => None,
            Self::TryFromNumber(error) => Some(error),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::IO(error)
    }
}

impl From<csv::Error> for Error {
    fn from(error: csv::Error) -> Self {
        Self::CSV(error)
    }
}

impl From<binance_client::errors::Error> for Error {
    fn from(error: binance_client::errors::Error) -> Self {
        Self::BinanceClient(error)
    }
}

impl From<diesel::result::Error> for Error {
    fn from(error: diesel::result::Error) -> Self {
        Self::Diesel(error)
    }
}

impl From<num::TryFromIntError> for Error {
    fn from(error: num::TryFromIntError) -> Self {
        Self::TryFromNumber(error)
    }
}

pub type Result<T = ()> = result::Result<T, Error>;
