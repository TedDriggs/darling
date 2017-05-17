use syn::Variant;

use Result;

/// Creates an instance from a specified `syn::Variant`.
pub trait FromVariant: Sized {
    /// Create an instance from `syn::Variant`, or return an error.
    fn from_variant(variant: &Variant) -> Result<Self>;
}

impl FromVariant for Variant {
    fn from_variant(variant: &Variant) -> Result<Self> {
        Ok(variant.clone())
    }
}