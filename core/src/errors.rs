pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(String);

impl Error {
    pub fn duplicate_field(name: &str) -> Self {
        unimplemented!()
    }

    pub fn missing_field(name: &str) -> Self {
        unimplemented!()
    }

    pub fn unknown_field(name: &str) -> Self {
        unimplemented!()
    }
}