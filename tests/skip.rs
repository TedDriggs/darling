//! Test that skipped fields are not read into structs when they appear in input.

#[macro_use]
extern crate darling;
extern crate syn;

use darling::FromDeriveInput;

#[derive(Debug, PartialEq, Eq, FromDeriveInput)]
#[darling(attributes(skip_test))]
pub struct Lorem {
    ipsum: String,

    #[darling(skip)]
    dolor: u8,
}

#[test]
fn verify_skipped_field_not_required() {
    let di = syn::parse_derive_input(r#"
        #[skip_test(ipsum = "Hello")]
        struct Baz;
    "#).unwrap();

    assert_eq!(Lorem::from_derive_input(&di).unwrap(), Lorem {
        ipsum: "Hello".to_string(),
        dolor: 0,
    });
}