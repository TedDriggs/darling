//! Fields *without* `#[darling(with = ...)]` route a right-hand side that fails to parse as a
//! `syn::Expr` to `FromMeta::from_invalid_expr`.
//!
//! `examples/html.rs` demonstrates the same feature, but examples are not run by `cargo test`.

use darling::{ast::MetaNameValueInvalidExpr, FromDeriveInput, FromMeta};
use syn::parse_quote;

/// `pub(crate)` is a `syn::Visibility` but not a `syn::Expr`, so it only ever arrives
/// through `from_invalid_expr`.
#[derive(Debug)]
struct Vis(syn::Visibility);

impl FromMeta for Vis {
    fn from_invalid_expr(value: &MetaNameValueInvalidExpr) -> darling::Result<Self> {
        syn::parse2(value.value.clone())
            .map(Self)
            .map_err(Into::into)
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(example))]
struct Example {
    access: Vis,
}

#[test]
fn derived_field_calls_from_invalid_expr() {
    let input = Example::from_derive_input(&parse_quote! {
        #[example(access = pub(crate))]
        struct Example;
    })
    .unwrap();

    assert!(matches!(input.access.0, syn::Visibility::Restricted(_)));
}

/// The default `from_invalid_expr` hands back the stored parse error.
#[test]
fn default_impl_reports_the_parse_error() {
    #[derive(Debug, FromDeriveInput)]
    #[darling(attributes(example))]
    struct NoOverride {
        #[allow(dead_code)]
        access: String,
    }

    let err = NoOverride::from_derive_input(&parse_quote! {
        #[example(access = pub(crate))]
        struct Example;
    })
    .unwrap_err();

    assert_eq!(err.to_string(), "expected an expression at access");
}
