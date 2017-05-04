use syn::DeriveInput;

use Result;

/// Parse input to a proc-macro derive.
pub trait FromDeriveInput: Sized {
    fn from_derive_input(input: &DeriveInput) -> Result<Self>;
}