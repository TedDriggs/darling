use syn;

use {FromMetaItem, Result};

mod container;
mod field;
mod from_derive;
mod from_field;
mod variant;

pub use self::container::Container;
pub use self::field::Field;
pub use self::from_derive::FromDeriveInputContainer;
pub use self::from_field::FromFieldOptions;
pub use self::variant::Variant;

/// A default/fallback expression encountered in attributes during parsing.
#[derive(Debug, Clone)]
pub enum DefaultExpression {
    /// The value should be taken from the `default` instance of the containing struct.
    /// This is not valid in container options.
    Inherit,
    Explicit(syn::Path),
    Trait,
}

#[doc(hidden)]
impl FromMetaItem for DefaultExpression {
    fn from_word() -> Result<Self> {
        Ok(DefaultExpression::Trait)
    }

    fn from_string(lit: &str) -> Result<Self> {
        Ok(DefaultExpression::Explicit(syn::parse_path(lit).unwrap()))
    }
}

pub trait ParseAttribute: Sized {
    fn parse_attributes(mut self, attrs: &[syn::Attribute]) -> Result<Self> {
        for attr in attrs {
            if attr.name() == "darling" {
                parse_attr(attr, &mut self)?;
            }
        }

        Ok(self)
    }

    fn parse_nested(&mut self, mi: &syn::MetaItem) -> Result<()>;
}

fn parse_attr<T: ParseAttribute>(attr: &syn::Attribute, target: &mut T) -> Result<()> {
    if attr.is_sugared_doc {
        return Ok(())
    }

    match attr.value {
        syn::MetaItem::List(_, ref items) => {
            for item in items {
                if let syn::NestedMetaItem::MetaItem(ref mi) = *item {
                    target.parse_nested(mi)?;
                } else {
                    unimplemented!();
                }
            }

            Ok(())
        },
        _ => unimplemented!()
    }
}