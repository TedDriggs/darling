pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(String);

impl Error {
    pub fn duplicate_field(name: &str) -> Self {
        panic!("Encountered duplicate field `{}`", name);
    }

    pub fn missing_field(name: &str) -> Self {
        panic!("Missing required field `{}`", name);
    }

    pub fn unknown_field(name: &str) -> Self {
        panic!("Encountered unknown field `{}`", name);
    }

    pub fn unsupported_format(format: &str) -> Self {
        panic!("Encountered unsupported format `{}`", format);
    }
}