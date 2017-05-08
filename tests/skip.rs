#[macro_use]
extern crate darling;
extern crate syn;

#[derive(Debug, PartialEq, Eq, FromField)]
#[darling(attributes(skip_test))]
pub struct Lorem {
    ipsum: String,

    #[darling(skip)]
    dolor: u8,
}

#[test]
fn verify() {
    
}