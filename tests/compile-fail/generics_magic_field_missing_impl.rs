#[derive(darling::FromDeriveInput)]
#[darling(attributes(example))]
pub struct Example {
    // There is no FromGenerics impl for Vec<String>.
    generics: Vec<String>,
}

fn main() {}
