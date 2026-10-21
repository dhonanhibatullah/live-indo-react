use std::{error::Error, fmt, str::FromStr};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Format {
    Plain,
    Json,
}

impl Format {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::Json => "json",
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Format {
    type Err = ParseFormatError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "plain" => Ok(Self::Plain),
            "json" => Ok(Self::Json),
            _ => Err(ParseFormatError),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParseFormatError;

impl fmt::Display for ParseFormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("logger format must be plain or json")
    }
}

impl Error for ParseFormatError {}
