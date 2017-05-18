use std::error::Error as StdError;
use std::fmt;
use std::string::ToString;

/// An alias of `Result` specific to attribute parsing.
pub type Result<T> = ::std::result::Result<T, Error>;

/// An error encountered during attribute parsing.
#[derive(Debug)]
pub struct Error {
    msg: String,
    locations: Vec<String>,
}

impl Error {
    pub fn custom<T: fmt::Display>(msg: T) -> Self {
        Error {
            msg: msg.to_string(),
            locations: Vec::new(),
        }
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

    /// Adds a location to the error, such as a field or variant. 
    /// Locations must be added in reverse order of specificity.
    pub fn at<T: fmt::Display>(mut self, location: T) -> Self {
        self.locations.insert(0, location.to_string());
        self
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        &self.msg
    }

    fn cause(&self) -> Option<&StdError> {
        None
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)?;
        if !self.locations.is_empty() {
            write!(f, " at {}", self.locations.join("/"))?;
        }

        Ok(())
    }
}