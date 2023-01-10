use syn::Data;

use crate::Result;

/// Creates an instance by parsing an syn::Data.
pub trait FromData: Sized {
    fn from_data(data: &Data) -> Result<Self>;
}

impl FromData for () {
    fn from_data(_: &Data) -> Result<Self> {
        Ok(())
    }
}

impl FromData for Data {
    fn from_data(data: &Data) -> Result<Self> {
        Ok(data.clone())
    }
}
