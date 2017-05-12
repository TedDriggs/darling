extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

extern crate darling_core;

use proc_macro::TokenStream;
use syn::parse_derive_input;

use darling_core::{options, codegen};

#[proc_macro_derive(FromMetaItem, attributes(darling))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_derive_input(&input.to_string()).unwrap();
    
    let container = options::FmiOptions::new(&ast).unwrap();
    let trait_impl = codegen::TraitImpl::from(&container);
    let result = quote!(#trait_impl);

    result.parse().expect(&format!("Couldn't parse `{}` to tokens", result))
}

#[proc_macro_derive(FromDeriveInput, attributes(darling))]
pub fn derive_from_input(input: TokenStream) -> TokenStream {
    let ast = parse_derive_input(&input.to_string()).unwrap();
    
    let mut fdic = options::FdiOptions::new(&ast).unwrap();

    let result = match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(fields_in)) => {

            let mut fields = Vec::with_capacity(fields_in.len());
            for field_in in fields_in {
                match field_in.ident.as_ref().map(|v| v.as_ref()) {
                    Some("ident") => fdic.base.ident = Some("ident".into()),
                    Some("vis") => fdic.vis = Some("vis".into()),
                    Some("generics") => fdic.generics = Some("generics".into()),
                    Some("attrs") => fdic.base.attrs = Some("attrs".into()),
                    _ => fields.push(options::InputField::from_field(&field_in, Some(&fdic.base.container)).unwrap())
                }
            }

            let trait_impl = codegen::TraitImpl {
                fields: fields.iter().map(options::InputField::as_codegen_field).collect(),
                ..(&fdic.base.container).into()
            };

            let fdi_view = codegen::FromDeriveInputImpl {
                struct_impl: trait_impl,
                ..(&fdic).into()
            };
            
            quote!(#fdi_view)
        },
        bd => panic!("Unsupported body `{:?}`", bd)
    };

    result.parse().expect(&format!("Couldn't parse `{}` to tokens", result))
}

#[proc_macro_derive(FromField, attributes(darling))]
pub fn derive_field(input: TokenStream) -> TokenStream {
    let ast = parse_derive_input(&input.to_string()).unwrap();
    
    let fdic = options::FromFieldOptions::new(&ast).unwrap();
    let generator = codegen::FromFieldImpl::from(&fdic);
    let result = quote!(#generator);

    result.parse().expect(&format!("Couldn't parse `{}` to tokens", result))
}