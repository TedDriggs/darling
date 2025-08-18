//! This is a regression test for issue [#371](https://github.com/TedDriggs/darling/issues/371).

use darling::util::{Callable, Override};
use darling::FromAttributes;

#[derive(FromAttributes)]
#[darling(attributes(test))]
#[allow(dead_code)]
struct Test1 {
    func: Callable,
}

#[derive(FromAttributes)]
#[darling(attributes(test))]
struct Test2 {
    func: Override<Callable>,
}

#[test]
fn test_explicit_closure() {
    let attrs = [syn::parse_quote!(#[test(func = || 1 + 1)])];
    assert!(Test1::from_attributes(&attrs).is_ok());
    assert!(Test2::from_attributes(&attrs).is_ok());
}

#[test]
fn test_explicit_path() {
    let attrs = [syn::parse_quote!(#[test(func = foo::bar)])];
    assert!(Test1::from_attributes(&attrs).is_ok());
    assert!(Test2::from_attributes(&attrs).is_ok());
}

#[test]
fn test_inherit() {
    let attrs = [syn::parse_quote!(#[test(func)])];
    assert!(Test1::from_attributes(&attrs).is_err());
    assert!(matches!(
        Test2::from_attributes(&attrs).unwrap().func,
        Override::Inherit
    ));
}
