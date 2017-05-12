//! When input is split across multiple attributes on one element,
//! darling should collapse that into one struct.

#[macro_use]
extern crate darling;
extern crate syn;

use std::string::ToString;

use darling::{Error, FromDeriveInput};

#[derive(Debug, FromDeriveInput, PartialEq, Eq)]
#[darling(attributes(split))]
struct Lorem {
    foo: String,
    bar: bool,
}

#[test]
fn split_attributes_accrue_to_instance() {
    let di = syn::parse_derive_input(r#"
        #[split(foo = "Hello")]
        #[split(bar)]
        pub struct Foo;
    "#).unwrap();

    let parsed = Lorem::from_derive_input(&di).unwrap();
    assert_eq!(parsed, Lorem {
        foo: "Hello".to_string(),
        bar: true,
    });
}

#[test]
fn duplicates_across_split_attrs_error() {
    let di = syn::parse_derive_input(r#"
        #[split(foo = "Hello")]
        #[split(foo = "World")]
        pub struct Foo;
    "#).unwrap();

    let pr = Lorem::from_derive_input(&di);
    assert_eq!(pr.unwrap_err().to_string(), Error::duplicate_field("foo").to_string());
}