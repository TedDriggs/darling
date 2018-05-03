use syn::Generics;

use {FromTypeParam, Result};

/// Creates an instance by parsing an entire generics declaration, including the
/// `where` clause.
pub trait FromGenerics: Sized {
    fn from_generics(generics: &Generics) -> Result<Self>;
}

impl FromGenerics for () {
    fn from_generics(_generics: &Generics) -> Result<Self> {
        Ok(())
    }
}

impl FromGenerics for Generics {
    fn from_generics(generics: &Generics) -> Result<Self> {
        Ok(generics.clone())
    }
}

impl<T: FromTypeParam> FromGenerics for Vec<T> {
    fn from_generics(generics: &Generics) -> Result<Self> {
        generics
            .type_params()
            .map(FromTypeParam::from_type_param)
            .collect()
    }
}
