//! This example shows how to parse completely arbitrary input (a self-closing HTML tag) -
//! via implementing [`FromMeta::from_verbatim`]

use syn::{Ident, Token};

fn main() {
    let input = quote::quote! {
        #[args(tag = <br />, foo = 10)]
        struct Input;
    };
    let input = syn::parse2::<syn::DeriveInput>(input).unwrap();
    let input = <Input as darling::FromDeriveInput>::from_derive_input(&input).unwrap();

    assert_eq!(
        input.tag,
        TagName(Ident::new("br", proc_macro2::Span::call_site()))
    );
    assert_eq!(input.foo, 10);
}

#[derive(Debug, PartialEq, Eq)]
struct TagName(Ident);

impl syn::parse::Parse for TagName {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![<]>()?;
        let tag_name = input.parse::<Ident>()?;
        input.parse::<Token![/]>()?;
        input.parse::<Token![>]>()?;

        Ok(Self(tag_name))
    }
}

impl darling::FromMeta for TagName {
    fn from_verbatim(tokens: &proc_macro2::TokenStream) -> Option<darling::Result<Self>> {
        Some(syn::parse2(tokens.clone()).map_err(Into::into))
    }
}

#[derive(darling::FromDeriveInput)]
#[darling(attributes(args))]
struct Input {
    tag: TagName,
    foo: usize,
}
