use syn;

use {FromMetaItem, Result};

mod container;
mod field;

pub use self::container::Container;
pub use self::field::Field;

/// A default/fallback expression encountered in attributes during parsing.
pub enum DefaultExpression {
    /// The value should be taken from the `default` instance of the containing struct.
    /// This is not valid in container options.
    InheritFromStruct,
    Explicit(syn::Path),
    Trait,
}

impl FromMetaItem for DefaultExpression {
    fn from_word() -> Result<Self> {
        Ok(DefaultExpression::Trait)
    }

    fn from_value(lit: syn::Lit) -> Result<Self> {
        match lit {
            syn::Lit::Str(s, _) => Ok(DefaultExpression::Explicit(syn::parse_path(&s).unwrap())),
            _ => panic!("Don't support non-strings in defaults"),
        }
    }
}