//! We used to have a problem where fields marked with `#[darling(with)]` or `#[darling(map)]`
//! would fail to compile if they do not implement `FromMeta`, even though they don't need to
//!
//! Issue: https://github.com/TedDriggs/darling/issues/305

use darling::FromDeriveInput;
use quote::quote;
use syn::DeriveInput;

fn demo(_: &syn::Meta) -> darling::Result<Vec<usize>> {
    Ok(Vec::from([4]))
}

fn parse_strings(_: String) -> Vec<String> {
    Vec::from(["x".to_string()])
}

#[derive(FromDeriveInput)]
#[darling(attributes(example))]
pub struct Example {
    #[darling(with = demo)]
    field: Vec<usize>,
    #[darling(map = parse_strings)]
    strings: Vec<String>,
}

#[test]
fn test() {
    let input = quote! {
        #[example(field, strings = "")]
        struct Example;
    };
    let input = syn::parse2::<DeriveInput>(input).unwrap();
    let input = Example::from_derive_input(&input).unwrap();
    assert_eq!(input.field, Vec::from([4]));
    assert_eq!(input.strings, Vec::from(["x".to_string()]));
}
