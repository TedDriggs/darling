fn wrong(_: syn::Ident) -> darling::Result<String> {
    Ok(String::new())
}

#[derive(darling::FromField)]
struct Input {
    #[darling(with = wrong)]
    ident: syn::Ident,
}

fn main() {}
