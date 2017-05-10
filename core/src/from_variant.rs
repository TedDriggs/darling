use syn::Variant;

use Result;

pub trait FromVariant {
    fn from_variant(variant: &Variant) -> Result<Self>;
}