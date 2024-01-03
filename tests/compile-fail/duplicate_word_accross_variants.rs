use darling::FromMeta;

#[derive(Debug, FromMeta, PartialEq, Eq)]
enum Choice {
    #[darling(word)]
    A,
    #[darling(word)]
    B,
    C,
}

fn main() {}
