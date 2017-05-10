use ident_case;
use syn::{self, Lit, MetaItem, NestedMetaItem};

use {Error, Result};

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
            Lit::Str(ref s, _) => Self::from_string(s),
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

impl FromMetaItem for () {
    fn from_word() -> Result<Self> {
        Ok(())
    }
}

impl FromMetaItem for bool {
    fn from_word() -> Result<Self> {
        Ok(true)
    }

    fn from_bool(value: bool) -> Result<Self> {
        Ok(value)
    }

    fn from_string(value: &str) -> Result<Self> {
        value.parse().or_else(|_| Err(Error::unknown_value(value)))
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
        syn::parse_path(value).or_else(|_| Err(Error::unknown_value(value)))
    }
}

impl FromMetaItem for syn::TyParamBound {
    fn from_string(value: &str) -> Result<Self> {
        syn::parse_ty_param_bound(value).or_else(|_| Err(Error::unknown_value(value)))
    }
}

impl FromMetaItem for syn::MetaItem {
    fn from_meta_item(value: &syn::MetaItem) -> Result<Self> {
        Ok(value.clone())
    }
}

impl FromMetaItem for ident_case::RenameRule {
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

impl<T: FromMetaItem> FromMetaItem for Result<T> {
    fn from_meta_item(item: &MetaItem) -> Result<Self> {
        Ok(FromMetaItem::from_meta_item(item))
    }
}

#[cfg(test)]
mod tests {
    use syn;
    
    use {FromMetaItem};

    /// parse a string as a syn::MetaItem instance.
    fn pmi(s: &str) -> ::std::result::Result<syn::MetaItem, String> {
        Ok(syn::parse_outer_attr(&format!("#[{}]", s))?.value)
    }

    fn fmi<T: FromMetaItem>(s: &str) -> T {
        FromMetaItem::from_meta_item(&pmi(s).expect("Tests should pass well-formed input"))
            .expect("Tests should pass valid input")
    }

    #[test]
    fn unit_succeeds() {
        assert_eq!(fmi::<()>("hello"), ());
    }

    #[test]
    fn bool_succeeds() {
        // word format
        assert_eq!(fmi::<bool>("hello"), true);

        // bool literal
        assert_eq!(fmi::<bool>("hello = true"), true);
        assert_eq!(fmi::<bool>("hello = false"), false);

        // string literals
        assert_eq!(fmi::<bool>(r#"hello = "true""#), true);
        assert_eq!(fmi::<bool>(r#"hello = "false""#), false);
    }

    #[test]
    fn string_succeeds() {
        // cooked form
        assert_eq!(&fmi::<String>(r#"hello = "world""#), "world");

        // raw form
        assert_eq!(&fmi::<String>(r##"hello = r#"world"#"##), "world");
    }

    #[test]
    fn meta_item_succeeds() {
        use syn::MetaItem;

        assert_eq!(fmi::<MetaItem>("hello(world,today)"), pmi("hello(world,today)").unwrap());
    }
}