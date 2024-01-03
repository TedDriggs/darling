use darling::FromMeta;

#[derive(Debug, FromMeta, PartialEq, Eq)]
enum Choice {
    #[darling(word, word)]
    A,
    B,
}

fn main() {}
