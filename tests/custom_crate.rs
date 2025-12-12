#![allow(
    unused,
    reason = "the fact that this test compiles is enough for it to pass"
)]

// Override the default location where we search
mod darling {}

use ::darling as renamed_darling;

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

fn main() {}
