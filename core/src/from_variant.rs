use syn::Variant;

use Result;

pub trait FromVariant: Sized {
    fn from_variant(variant: &Variant) -> Result<Self>;
}