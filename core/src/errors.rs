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
}