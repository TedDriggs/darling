use serde_case;
use syn::{self, Lit, MetaItem, NestedMetaItem};

use {Error, Result};

/// Mutate an instance by applying declarations in an attribute declaration.
pub trait ApplyMetaItem: Sized {
    fn from_list(&mut self, items: &[NestedMetaItem]) -> Result<&mut Self>;
}

/// Create an instance from an item in an attribute declaration. 
/// 
/// # Implementing `FromMetaItem`
/// * Do not take a dependency on the `ident` of the passed-in meta item. The ident will be set by the field name of the containing struct.
/// * Implement only the `from_*` methods that you intend to support. The default implementations will return useful errors.
///
/// # Provided Implementations
/// ## bool
/// 
/// * Word with no value specified - becomes `true`.
/// * As a boolean literal, e.g. `foo = true`.
/// * As a string literal, e.g. `foo = "true"`.
pub trait FromMetaItem: Sized {
    fn from_nested_meta_item(item: &NestedMetaItem) -> Result<Self> {
        match *item {
            NestedMetaItem::Literal(ref lit) => Self::from_value(lit),
            NestedMetaItem::MetaItem(ref mi) => Self::from_meta_item(mi),
        }
    }

    /// Create an instance from a `syn::MetaItem` by dispatching to the format-appropriate
    /// trait function. This generally should not be overridden by implementers.
    fn from_meta_item(item: &MetaItem) -> Result<Self> {
        match *item {
            MetaItem::Word(_) => Self::from_word(),
            MetaItem::List(_, ref items) => Self::from_list(items),
            MetaItem::NameValue(_, ref val) => Self::from_value(val),
        }
    }

    /// Create an instance from the presence of the word in the attribute with no
    /// additional options specified.
    fn from_word() -> Result<Self> {
        Err(Error::unsupported_format("word"))
    }

    /// Create an instance from a list of nested meta items.
    #[allow(unused_variables)]
    fn from_list(items: &[NestedMetaItem]) -> Result<Self> {
        Err(Error::unsupported_format("list"))
    }

    /// Create an instance from a literal value of either `foo = "bar"` or `foo("bar")`.
    /// This dispatches to the appropriate method based on the type of literal encountered,
    /// and generally should not be overridden by implementers.
    #[allow(unused_variables)]
    fn from_value(value: &Lit) -> Result<Self> {
        match *value {
            Lit::Bool(ref b) => Self::from_bool(b.clone()),
            Lit::Str(ref s, syn::StrStyle::Cooked) => Self::from_string(s),
            ref _other => Err(Error::unexpected_type("other"))
        }
    }

    /// Create an instance from a string literal in a value position.
    #[allow(unused_variables)]
    fn from_string(value: &str) -> Result<Self> {
        Err(Error::unexpected_type("string"))
    }

    /// Create an instance from a bool literal in a value position.
    #[allow(unused_variables)]
    fn from_bool(value: bool) -> Result<Self> {
        Err(Error::unexpected_type("bool"))
    }
}

// FromMetaItem impls for std and syn types.

impl FromMetaItem for bool {
    fn from_word() -> Result<Self> {
        Ok(true)
    }

    fn from_bool(value: bool) -> Result<Self> {
        Ok(value)
    }

    fn from_string(value: &str) -> Result<Self> {
        Ok(value.parse().unwrap())
    }
}

impl FromMetaItem for String {
    fn from_string(s: &str) -> Result<Self> {
        Ok(s.to_string())
    }
}

impl FromMetaItem for syn::Ident {
    fn from_string(value: &str) -> Result<Self> {
        Ok(syn::Ident::new(value))
    }
}

impl FromMetaItem for syn::Path {
    fn from_string(value: &str) -> Result<Self> {
        Ok(syn::parse_path(value).unwrap())
    }
}

impl FromMetaItem for syn::TyParamBound {
    fn from_string(value: &str) -> Result<Self> {
        Ok(syn::parse_ty_param_bound(value).unwrap())
    }
}

impl FromMetaItem for serde_case::RenameRule {
    fn from_string(value: &str) -> Result<Self> {
        value.parse().or_else(|_| Err(Error::unknown_value(value)))
    }
}

impl<T: FromMetaItem> FromMetaItem for Option<T> {
    fn from_meta_item(item: &MetaItem) -> Result<Self> {
        Ok(Some(FromMetaItem::from_meta_item(item)?))
    }
}

impl<T: FromMetaItem> FromMetaItem for Box<T> {
    fn from_meta_item(item: &MetaItem) -> Result<Self> {
        Ok(Box::new(FromMetaItem::from_meta_item(item)?))
    }
}