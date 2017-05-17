#[macro_use]
extern crate darling;
extern crate syn;

#[derive(FromVariant)]
#[darling(from_ident, attributes(hello))]
pub struct Lorem {
    ident: syn::Ident,
    into: Option<bool>,
    skip: Option<bool>,
    data: darling::util::VariantData<syn::Ty>,
}

#[test]
fn expansion() {
    
}