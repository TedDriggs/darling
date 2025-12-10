#[derive(darling::FromDeriveInput)]
#[darling(attributes(example))]
pub struct Example {
    // There is no TryFrom<&syn::Data> impl for Vec<String>.
    data: Vec<String>,
}

fn main() {}
