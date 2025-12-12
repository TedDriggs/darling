fn wrong(_: proc_macro2::Ident) -> darling::Result<()> {
    Ok(())
}

#[derive(darling::FromField)]
struct Input {
    #[darling(with = wrong)]
    ident: (),
}

fn main() {}
