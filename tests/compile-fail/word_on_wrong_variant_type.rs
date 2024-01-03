use darling::FromMeta;

#[derive(Debug, FromMeta, PartialEq, Eq)]
enum Meta {
    Unit,
    #[darling(word)]
    NotUnit(String)
}

fn main() {}
