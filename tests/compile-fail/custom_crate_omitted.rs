mod darling {}

#[derive(darling::FromDeriveInput)]
// forgot to specify #[darling(crate = darling)]
pub struct Example {
    ident: Option<syn::Ident>,
}

fn main() {}
