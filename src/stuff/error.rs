use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use reqwest::StatusCode;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Request(reqwest::Error),
    FailedToGetNewMessage(StatusCode, String),
    OrderNotFound(String),
    PaperInvalid,
    SizeInvalid(String),
    OrderWrongState,
    ParseFailed(ParseIntError),
    OrderFailed(String)
}

// region:    ---From
impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Error {
        Error::ParseFailed(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Request(err)
    }
}

// endregion: ---From

// region:    --- Error boilerplate
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
// endregion: --- Error boilerplate