//! `and_then` and `map` should remote the requirement to implement `FromMeta`
#![allow(dead_code)]

use darling::{FromDeriveInput, Result};
use syn::parse_quote;

#[derive(Debug)]
struct DoesNotImplFromMeta;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(hello))]
struct Foo {
    #[darling(map = "map_fn")]
    map: DoesNotImplFromMeta,
    #[darling(and_then = "and_then_fn")]
    and_then: DoesNotImplFromMeta,
    #[darling(map = "opt_map_fn")]
    opt_map: Option<DoesNotImplFromMeta>,
    #[darling(and_then = "opt_and_then_fn")]
    opt_and_then: Option<DoesNotImplFromMeta>,
}

fn map_fn(_: syn::Lit) -> DoesNotImplFromMeta {
    DoesNotImplFromMeta
}

fn and_then_fn(_: syn::Expr) -> Result<DoesNotImplFromMeta> {
    Ok(DoesNotImplFromMeta)
}

fn opt_map_fn(opt: Option<syn::Ident>) -> Option<DoesNotImplFromMeta> {
    opt.map(|_| DoesNotImplFromMeta)
}

fn opt_and_then_fn(opt: Option<syn::Path>) -> Result<Option<DoesNotImplFromMeta>> {
    Ok(opt.map(|_| DoesNotImplFromMeta))
}

#[test]
fn expansion() {
    let di = parse_quote! {
        #[hello(map = false, and_then = "42")]
        #[hello(opt_map = "foo")]
        pub struct Foo;
    };

    Foo::from_derive_input(&di).unwrap();
}
