use syn::Field;

use Result;

/// Creates an instance by parsing an individual field and its attributes.
pub trait FromField: Sized {
    fn from_field(field: &Field) -> Result<Self>;
}