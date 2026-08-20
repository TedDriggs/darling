//! A field with `#[darling(with = ...)]` must not require its type to impl `FromMeta`,
//! including when the right-hand side of the input fails to parse as a `syn::Expr`.
//!
//! This is the invalid-expression counterpart to `with_no_impl_from_meta.rs`.
//!
//! Issue: https://github.com/TedDriggs/darling/issues/433

use darling::{FromDeriveInput, FromMeta};
use syn::parse_quote;

/// Deliberately does not impl `FromMeta`; `with` is what makes it parseable.
#[derive(Debug, Default, PartialEq, Eq)]
struct NotFromMeta(String);

fn parser(meta: &syn::Meta) -> darling::Result<NotFromMeta> {
    String::from_meta(meta).map(NotFromMeta)
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(example))]
struct Example {
    #[darling(default, with = parser)]
    with: NotFromMeta,
}

/// The shape from the issue: the same field on a `FromMeta` derive.
#[derive(FromMeta)]
struct Opts {
    #[darling(default, with = parser)]
    with: NotFromMeta,
}

#[test]
fn parses_via_the_with_callable() {
    let input: Example = Example::from_derive_input(&parse_quote! {
        #[example(with = "hello")]
        struct Example;
    })
    .unwrap();

    assert_eq!(input.with, NotFromMeta("hello".to_string()));
}

#[test]
fn missing_input_uses_default() {
    let input: Example = Example::from_derive_input(&parse_quote! {
        #[example]
        struct Example;
    })
    .unwrap();

    assert_eq!(input.with, NotFromMeta::default());
}

#[test]
fn parses_via_the_with_callable_on_from_meta() {
    let opts = Opts::from_meta(&parse_quote!(example(with = "hello"))).unwrap();

    assert_eq!(opts.with, NotFromMeta("hello".to_string()));
}

/// A `with` callable only accepts a `syn::Meta`, so a right-hand side that isn't a valid
/// expression has to come back as an error rather than panicking or reaching
/// `FromMeta::from_invalid_expr`, which the field type doesn't implement.
#[test]
fn invalid_expr_is_an_error() {
    let err = Example::from_derive_input(&parse_quote! {
        #[example(with = pub(crate))]
        struct Example;
    })
    .unwrap_err();

    // The stored `syn::Expr` parse error, located at the field it came from.
    assert_eq!(err.to_string(), "expected an expression at with");
}
