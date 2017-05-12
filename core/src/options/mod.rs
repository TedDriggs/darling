use syn;

use {FromMetaItem, Result, Error};
use util;

mod core;
mod forward_attrs;
mod from_derive;
mod from_field;
mod from_meta_item;
mod from_variant;
mod input_variant;
mod input_field;
mod outer_from;

pub use self::core::Core;
pub use self::forward_attrs::ForwardAttrs;
pub use self::from_derive::FdiOptions;
pub use self::from_field::FromFieldOptions;
pub use self::from_meta_item::FmiOptions;
pub use self::from_variant::FromVariantOptions;
pub use self::input_variant::InputVariant;
pub use self::input_field::InputField;
pub use self::outer_from::OuterFrom;

pub type Body = util::Body<InputVariant, InputField>;

/// A default/fallback expression encountered in attributes during parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// Middleware for extracting attribute values.
pub trait ParseAttribute: Sized {
    fn parse_attributes(mut self, attrs: &[syn::Attribute]) -> Result<Self> {
        for attr in attrs {
            if attr.name() == "darling" {
                parse_attr(attr, &mut self)?;
            }
        }

        Ok(self)
    }

    /// Read a meta-item, and apply its values to the current instance.
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

pub trait ParseBody: Sized {
    fn parse_body(mut self, body: &syn::Body) -> Result<Self> {
        use syn::{Body, VariantData};

        match *body {
            Body::Struct(VariantData::Unit) => Ok(self),
            Body::Struct(VariantData::Struct(ref fields)) => {
                for field in fields {
                    self.parse_field(field)?;
                }

                Ok(self)
            },
            Body::Struct(_) => unimplemented!(),
            Body::Enum(_) => unimplemented!(),
        }
    }

    #[allow(unused_variables)]
    fn parse_variant(&mut self, variant: &syn::Variant) -> Result<()> {
        Err(Error::unsupported_format("enum variant"))
    }

    #[allow(unused_variables)]
    fn parse_field(&mut self, field: &syn::Field) -> Result<()> {
        Err(Error::unsupported_format("struct field"))
    }
}