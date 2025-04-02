use std::fmt::{Display, Formatter};
use crate::stuff;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    // -- Config
    ConfigMissingEnv(&'static str),
    ConfigWrongFormat(&'static str),
    App(stuff::error::Error)
}

//region      --- From
impl From<stuff::error::Error> for Error {
    fn from(value: stuff::error::Error) -> Self {
        Error::App(value)
    }
}
//endregion   --- From

// region:    --- Error boilerplate
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
// endregion: --- Error boilerplate