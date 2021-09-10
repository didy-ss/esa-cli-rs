use std::error;
use std::fmt;
use std::io;
use toml;

#[derive(Debug)]
pub enum Error {
    EsaInfo(toml::de::Error),
    PostNameIsInvalid,
    SavePost(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::EsaInfo(msg) => write!(f, "Set Environment Variables: {}", msg),
            Error::PostNameIsInvalid => write!(f, "file name must be ([category/]+)filename"),
            Error::SavePost(msg) => write!(f, "{}", msg),
        }
    }
}

impl error::Error for Error {}
