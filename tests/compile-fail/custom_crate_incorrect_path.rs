#[derive(darling::FromDeriveInput)]
#[darling(crate = not_darling)]
pub struct Example {
    ident: Option<syn::Ident>,
}

fn main() {}
