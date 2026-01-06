//! Transparent structs delegate their implementation to the inner type
//!
//! A single-field tuple variant or struct like `Foo(bar)` is transparent by default.

use darling::{FromDeriveInput, FromMeta};
use syn::parse_quote;

#[derive(Debug, FromMeta, PartialEq, Eq)]
struct Lorem(bool);

#[derive(Debug, FromMeta, PartialEq, Eq)]
#[darling(transparent)]
struct Lorem2 {
    named: bool,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(newtype))]
struct DemoContainer {
    lorem: Lorem,
    lorem_2: Lorem2,
}

#[derive(Debug, FromDeriveInput)]
struct Outer(DemoContainer);

#[derive(Debug, FromDeriveInput)]
#[darling(transparent)]
struct Outer2 {
    inner: DemoContainer,
}

#[test]
fn generated() {
    let di = parse_quote! {
        #[derive(Baz)]
        #[newtype(lorem = false, lorem_2 = false)]
        pub struct Foo;
    };

    let c = Outer::from_derive_input(&di).unwrap();
    let c2 = Outer2::from_derive_input(&di).unwrap();

    assert_eq!(c.0.lorem, Lorem(false));
    assert_eq!(c.0.lorem_2, Lorem2 { named: false });
    assert_eq!(c2.inner.lorem, Lorem(false));
    assert_eq!(c2.inner.lorem_2, Lorem2 { named: false });
}
