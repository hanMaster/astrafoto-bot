use std::fmt::{Display, Formatter};

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    // -- Config
    ConfigMissingEnv(&'static str),
}

// region:    --- Error boilerplate
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
// endregion: --- Error boilerplate