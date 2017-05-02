#![recursion_limit = "256"]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate quote;

extern crate syn;

use syn::{NestedMetaItem, MetaItem, Lit};

pub mod codegen;
mod errors;
pub mod options;

pub use errors::{Result, Error};

/// Create an instance from an item in an attribute declaration.
pub trait FromMetaItem: Sized {
    fn from_nested_meta_item(item: NestedMetaItem) -> Result<Self> {
        match item {
            NestedMetaItem::Literal(lit) => Self::from_value(lit),
            NestedMetaItem::MetaItem(mi) => Self::from_meta_item(mi),
        }
    }

    /// Create an instance from a `syn::MetaItem` by dispatching to the format-appropriate
    /// trait function. This generally should not be overridden by implementers.
    fn from_meta_item(item: MetaItem) -> Result<Self> {
        match item {
            MetaItem::Word(_) => Self::from_word(),
            MetaItem::List(_, items) => Self::from_list(items),
            MetaItem::NameValue(_, val) => Self::from_value(val),
        }
    }

    fn from_word() -> Result<Self> {
        Err(Error::unsupported_format("word"))
    }

    #[allow(unused_variables)]
    fn from_list(items: Vec<NestedMetaItem>) -> Result<Self> {
        Err(Error::unsupported_format("list"))
    }

    #[allow(unused_variables)]
    fn from_value(value: Lit) -> Result<Self> {
        Err(Error::unsupported_format("value"))
    }
}

impl FromMetaItem for bool {
    fn from_word() -> Result<Self> {
        Ok(true)
    }

    fn from_value(value: Lit) -> Result<Self> {
        match value {
            Lit::Bool(b) => Ok(b),
            Lit::Str(s, _) => Ok(s.parse().unwrap()),
            _ => panic!("Only bools and strings can parse to bools"),
        }
    }
}

impl FromMetaItem for String {
    fn from_value(value: Lit) -> Result<Self> {
        match value {
            Lit::Str(s, _) => Ok(s),
            _ => unimplemented!()
        }
    }
}

impl FromMetaItem for syn::Ident {
    fn from_value(value: Lit) -> Result<Self> {
        match value {
            Lit::Str(s, _) => Ok(syn::Ident::new(s)),
            _ => unimplemented!(),
        }
    }
}

impl FromMetaItem for syn::Path {
    fn from_value(value: Lit) -> Result<Self> {
        match value {
            Lit::Str(ref k, _) => Ok(syn::parse_path(k).unwrap()),
            _ => unimplemented!(),
        }
    }
}

impl<T: FromMetaItem> FromMetaItem for Option<T> {
    fn from_meta_item(item: MetaItem) -> Result<Self> {
        Ok(Some(FromMetaItem::from_meta_item(item)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn do_things() {
        let foo = options::Field {
            target_name: syn::parse_ident("lorem").unwrap(),
            attr_name: Some(syn::parse_ident("ipsum").unwrap()),
            ty: syn::parse_type("bool").unwrap(),
            default: None,
            with: None,
        };

        let bar = options::Field {
            target_name: syn::parse_ident("dolor").unwrap(),
            attr_name: None,
            ty: syn::parse_type("String").unwrap(),
            default: None,
            with: None,
        };

        let container = options::Container {
            ident: syn::parse_ident("Foo").unwrap(),
            generics: Default::default(),
            default: None,
        };

        let derive_tgt = codegen::TraitImpl {
            fields: vec![foo.as_codegen_field(), bar.as_codegen_field()],
            ..(&container).into()
        };

        println!("{}", quote!(#derive_tgt));
    }
}