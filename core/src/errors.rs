use std::fmt;

/// An alias of `Result` specific to attribute parsing.
pub type Result<T> = ::std::result::Result<T, Error>;

/// An error encountered during attribute parsing.
#[derive(Debug)]
pub struct Error(String);

impl Error {
    pub fn duplicate_field(name: &str) -> Self {
        Error(format!("Encountered duplicate field `{}`", name))
    }

    pub fn missing_field(name: &str) -> Self {
        Error(format!("Missing required field `{}`", name))
    }

    pub fn unknown_field(name: &str) -> Self {
        Error(format!("Encountered unknown field `{}`", name))
    }

    pub fn unsupported_format(format: &str) -> Self {
        Error(format!("Encountered unsupported format `{}`", format))
    }

    pub fn unexpected_type(ty: &str) -> Self {
        Error(format!("Unexpected literal type `{}`", ty))
    }

    pub fn unknown_value(value: &str) -> Self {
        Error(format!("Encountered unknown value `{}`", value))
    }

    pub fn too_few_values(min: usize) -> Self {
        Error(format!("Didn't get enough values; expected {}", min))
    }

    pub fn too_many_values(max: usize) -> Self {
        Error(format!("Got too many values; expected no more than {}", max))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}