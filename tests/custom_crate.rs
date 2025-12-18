#![allow(
    unused,
    reason = "the fact that this test compiles is enough for it to pass"
)]

// Override the default location where we search
mod darling {}

use ::darling::{self as renamed_darling, FromDeriveInput};
use quote::quote;
use syn::DeriveInput;

mod inner {
    pub use ::darling as renamed_darling;
}

#[derive(::darling::FromDeriveInput)]
#[darling(crate = renamed_darling)]
struct First {
    ident: syn::Ident,
}

#[derive(::darling::FromDeriveInput)]
#[darling(crate = inner::renamed_darling)]
struct Second {
    ident: syn::Ident,
}

#[test]
fn renamed_darling() {
    let input = quote! {
        struct FirstInput {
            field: ()
        }
    };

    let input = syn::parse2(input).unwrap();
    let input = First::from_derive_input(&input).unwrap();
    assert_eq!(input.ident, "FirstInput");
}

#[test]
pub fn renamed_darling_in_module() {
    let input = quote! {
        struct SecondInput {
            field: ()
        }
    };

    let input = syn::parse2(input).unwrap();
    let input = Second::from_derive_input(&input).unwrap();
    assert_eq!(input.ident, "SecondInput");
}
