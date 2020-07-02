use serde_json;
use std::io;

#[derive(Debug)]
pub enum Error {
    SerdeError(serde_json::Error),
    IoError(io::Error),
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerdeError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError(err)
    }
}
