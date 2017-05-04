use syn::Field;

use Result;

pub trait FromField {
    fn from_field(field: &Field) -> Result<Self>;
}