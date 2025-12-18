// Override the default location where we search
mod darling {}

use ::darling::{self as renamed_darling, FromDeriveInput};
use quote::quote;

mod inner {
    pub use ::darling as renamed_darling;
}

// Renamed to a single module, also test `derive_syn_parse`
#[derive(::darling::FromMeta)]
#[darling(crate = renamed_darling, derive_syn_parse)]
struct InputAttr {
    foo: ::darling::util::Flag,
}

// Renamed to a nested module
#[derive(::darling::FromDeriveInput)]
#[darling(attributes(input), crate = inner::renamed_darling)]
struct Input {
    ident: syn::Ident,
    my_meta: InputAttr,
}

#[test]
fn renamed_darling() {
    let input = quote! {
        #[input(my_meta(foo))]
        struct FirstInput {
            field: ()
        }
    };

    let input = syn::parse2(input).unwrap();
    let input = Input::from_derive_input(&input).unwrap();
    assert_eq!(input.ident, "FirstInput");
    assert!(input.my_meta.foo.is_present());
}

#[test]
fn renamed_darling_syn() {
    let input = syn::parse2::<InputAttr>(quote! { foo }).unwrap();
    assert!(input.foo.is_present());
}
