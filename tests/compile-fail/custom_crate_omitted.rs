use ::darling as not_darling;

mod darling {}

#[derive(not_darling::FromDeriveInput)]
// forgot to specify #[darling(crate = not_darling)]
pub struct Example {
    ident: Option<syn::Ident>,
}

fn main() {}
