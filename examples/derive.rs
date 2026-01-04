//! Shows how to implement a `#[derive]`-like macro.

use std::collections::HashSet;

use darling::FromDeriveInput;
use syn::{parse_quote, Path};

#[derive(FromDeriveInput, PartialEq, Eq)]
#[darling(attributes(derive))]
pub struct Derive {
    // Note that `impl FromMeta for HashSet<Path>` errors on duplicates
    #[darling(flatten)]
    derives: HashSet<Path>,
}

fn main() {
    let input = Derive::from_derive_input(&parse_quote! {
        #[derive(Debug, Copy, Clone, Eq, PartialEq, std::hash::Hash)]
        struct Example;
    })
    .unwrap();

    assert_eq!(
        input.derives,
        [
            parse_quote!(Debug),
            parse_quote!(Copy),
            parse_quote!(Clone),
            parse_quote!(Eq),
            parse_quote!(PartialEq),
            parse_quote!(std::hash::Hash),
        ]
        .into()
    );
}
