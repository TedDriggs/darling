use syn::{Field, Ty};

use Result;

/// Creates an instance by parsing an individual field and its attributes.
pub trait FromField: Sized {
    fn from_field(field: &Field) -> Result<Self>;
}

impl FromField for Field {
    fn from_field(field: &Field) -> Result<Self> {
        Ok(field.clone())
    }
}

impl FromField for Ty {
    fn from_field(field: &Field) -> Result<Self> {
        Ok(field.ty.clone())
    }
}