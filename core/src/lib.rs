#![recursion_limit = "256"]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate quote;

extern crate syn;

extern crate ident_case;

pub mod codegen;
mod errors;
mod from_field;
mod from_derive_input;
mod from_meta_item;
mod from_variant;
pub mod options;
pub mod util;

pub use errors::{Result, Error};
pub use from_derive_input::FromDeriveInput;
pub use from_field::FromField;
pub use from_meta_item::{FromMetaItem};
pub use from_variant::FromVariant;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn do_things() {
        let foo = options::MetaItemField {
            target_name: syn::parse_ident("lorem").unwrap(),
            attr_name: Some("ipsum".to_string()),
            ty: syn::parse_type("bool").unwrap(),
            default: None,
            with: None,
            skip: false,
            map: None,
        };

        let bar = options::MetaItemField {
            target_name: syn::parse_ident("dolor").unwrap(),
            attr_name: None,
            ty: syn::parse_type("String").unwrap(),
            default: None,
            with: None,
            skip: false,
            map: None,
        };

        let container = options::Core {
            ident: syn::parse_ident("Foo").unwrap(),
            generics: Default::default(),
            default: Default::default(),
            rename_rule: Default::default(),
            map: Default::default(),
        };

        let derive_tgt = codegen::TraitImpl {
            fields: vec![foo.as_codegen_field(), bar.as_codegen_field()],
            ..(&container).into()
        };

        println!("{}", quote!(#derive_tgt));
    }
}