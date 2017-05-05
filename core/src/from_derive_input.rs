use syn::DeriveInput;

use Result;

/// Creates an instance by parsing an entire proc-macro `derive` input,
/// including the, identity, generics, and visibility of the type.
pub trait FromDeriveInput: Sized {
    fn from_derive_input(input: &DeriveInput) -> Result<Self>;
}