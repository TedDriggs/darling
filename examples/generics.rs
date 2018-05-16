//! This example shows darling being used in more complex generic cases.
//!
//! 1. A generic receiver struct is used, with the type param having a custom bound.
//! 1. A generated impl of `FromTypeParam` is used to collect type params from the generics,
//!    including parsing a custom attribute on the type param.
//! 1. Magic fields of the type param are used to get the default and ident.
//!
//! For brevity (and due to lack of inspiration for fake use-cases), this example omits
//! code generation.

#[macro_use]
extern crate darling;

extern crate quote;
extern crate syn;

use darling::{ast, FromDeriveInput, FromTypeParam};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(lorem))]
struct LoremReceiver<#[darling(bound = "FromTypeParam")] T> {
    generics: ast::Generics<ast::GenericParam<T>>,
}

#[derive(Debug, FromTypeParam)]
#[darling(attributes(lorem))]
struct TypeParamReceiver {
    ident: syn::Ident,
    default: Option<syn::Type>,
    #[darling(default)]
    skip: bool,
}

fn main() {
    let input = r#"#[derive(MyTrait)]
pub struct Foo<#[lorem(skip)] T = ()> {
    bar: T,

    baz: i64,
}"#;

    let parsed = syn::parse_str(input).unwrap();
    let receiver: LoremReceiver<TypeParamReceiver> =
        FromDeriveInput::from_derive_input(&parsed).unwrap();

    println!(
        r#"
INPUT:

{}

PARSED AS:

{:?}
    "#,
        input, receiver
    );
}
