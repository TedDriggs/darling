use std::fmt;
use std::string::ToString;

/// An alias of `Result` specific to attribute parsing.
pub type Result<T> = ::std::result::Result<T, Error>;

/// An error encountered during attribute parsing.
#[derive(Debug)]
pub struct Error(String);

impl Error {
    pub fn custom<T: fmt::Display>(msg: T) -> Self {
        Error(msg.to_string())
    }

    pub fn duplicate_field(name: &str) -> Self {
        Error::custom(format!("Encountered duplicate field `{}`", name))
    }

    pub fn missing_field(name: &str) -> Self {
        Error::custom(format!("Missing required field `{}`", name))
    }

    pub fn unknown_field(name: &str) -> Self {
        Error::custom(format!("Encountered unknown field `{}`", name))
    }

    pub fn unsupported_format(format: &str) -> Self {
        Error::custom(format!("Encountered unsupported format `{}`", format))
    }

    pub fn unexpected_type(ty: &str) -> Self {
        Error::custom(format!("Unexpected literal type `{}`", ty))
    }

    pub fn unknown_value(value: &str) -> Self {
        Error::custom(format!("Encountered unknown value `{}`", value))
    }

    pub fn too_few_items(min: usize) -> Self {
        Error::custom(format!("Didn't get enough values; expected {}", min))
    }

    pub fn too_many_items(max: usize) -> Self {
        Error::custom(format!("Got too many values; expected no more than {}", max))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}