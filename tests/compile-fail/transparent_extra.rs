// The extra "foo" field is an error
use darling::FromMeta;

#[derive(Debug, FromMeta, PartialEq, Eq)]
#[darling(transparent)]
struct Lorem3 {
    named: bool,
    foo: bool,
}

fn main() {}
