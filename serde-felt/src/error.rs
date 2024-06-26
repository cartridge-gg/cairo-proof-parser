use std::fmt::{self, Display};

use serde::{de, ser};

#[derive(Debug)]
pub enum Error {
    Message(String),
    Error,
    DataLeft,
    NoDataLeft,
    InvalidArrayLen,
    ValueExceededRange,
    LengthSpecifiedButNotEnoughProvided,
    MoreLengthsThanVectors,
    LengthSetButNotConsumed,
    LengthNotKnownAtSerialization,
    UnparsableString,
}
pub type Result<T> = std::result::Result<T, Error>;

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
            Error::DataLeft => formatter.write_str("unexpected end of input"),
            Error::Error => formatter.write_str("Invalid proof hex"),
            Error::NoDataLeft => formatter.write_str("unexpected end of input"),
            Error::InvalidArrayLen => formatter.write_str("invalid array length"),
            Error::ValueExceededRange => formatter.write_str("value exceeded range"),
            Error::LengthSpecifiedButNotEnoughProvided => {
                formatter.write_str("length specified but not enough provided")
            }
            Error::MoreLengthsThanVectors => formatter.write_str("more lengths than vectors"),
            Error::LengthSetButNotConsumed => formatter.write_str("length set but not consumed"),
            Error::LengthNotKnownAtSerialization => {
                formatter.write_str("length not known at serialization")
            }
            Error::UnparsableString => formatter.write_str("non-parsable strings not supported"),
        }
    }
}

impl std::error::Error for Error {}
