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
    
    let container = options::Container::new(ast.ident, ast.generics, &ast.attrs).unwrap();

    let result = match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(fields_in)) => {
            let mut fields = Vec::with_capacity(fields_in.len());
            for field_in in fields_in {
                fields.push(options::Field::from_field(field_in, Some(&container)).unwrap());
            }

            let trait_impl = codegen::TraitImpl {
                fields: fields.iter().map(options::Field::as_codegen_field).collect(),
                ..(&container).into()
            };

            quote!(#trait_impl)
        },
        syn::Body::Enum(src_variants) => {
            let mut variants = Vec::with_capacity(src_variants.len());
            for src_var in src_variants {
                variants.push(options::Variant::from_variant(src_var, Some(&container)).unwrap());
            }

            let enum_impl = codegen::EnumImpl {
                variants: variants.iter()
                    .map(|v| v.as_codegen_variant(&container.ident))
                    .collect(),
                ..(&container).into()
            };

            quote!(#enum_impl)
        }
        bd => panic!("Unsupported body `{:?}`", bd)
    };

    result.parse().expect(&format!("Couldn't parse `{}` to tokens", result))
}

#[proc_macro_derive(FromDeriveInput, attributes(darling))]
pub fn derive_from_input(input: TokenStream) -> TokenStream {
    let ast = parse_derive_input(&input.to_string()).unwrap();
    
    let mut fdic = options::FromDeriveInputContainer::new(ast.ident, ast.generics, &ast.attrs).unwrap();

    let result = match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(fields_in)) => {

            let mut fields = Vec::with_capacity(fields_in.len());
            for field_in in fields_in {
                match field_in.ident.as_ref().map(|v| v.as_ref()) {
                    Some("ident") => fdic.ident = Some("ident".into()),
                    Some("vis") => fdic.vis = Some("vis".into()),
                    Some("generics") => fdic.generics = Some("generics".into()),
                    Some("attrs") => fdic.attrs = Some("attrs".into()),
                    _ => fields.push(options::Field::from_field(field_in, Some(&fdic.container)).unwrap())
                }
            }

            let trait_impl = codegen::TraitImpl {
                fields: fields.iter().map(options::Field::as_codegen_field).collect(),
                ..(&fdic.container).into()
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
    
    let mut fdic = options::FromFieldOptions::new(ast.ident, ast.generics, &ast.attrs).unwrap();
    
    let result = match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(fields_in)) => {

            let mut fields = Vec::with_capacity(fields_in.len());
            for field_in in fields_in {
                match field_in.ident.as_ref().map(|v| v.as_ref()) {
                    Some("ident") => fdic.ident = Some("ident".into()),
                    Some("vis") => fdic.vis = Some("vis".into()),
                    Some("ty") => fdic.ty = Some("ty".into()),
                    Some("attrs") => fdic.attrs = Some("attrs".into()),
                    _ => fields.push(options::Field::from_field(field_in, Some(&fdic.container)).unwrap())
                }
            }

            let trait_impl = codegen::TraitImpl {
                fields: fields.iter().map(options::Field::as_codegen_field).collect(),
                ..(&fdic.container).into()
            };

            let fdi_view = codegen::FromFieldImpl {
                body: trait_impl,
                ..(&fdic).into()
            };
            
            quote!(#fdi_view)
        },
        bd => panic!("Unsupported body `{:?}`", bd)
    };

    result.parse().expect(&format!("Couldn't parse `{}` to tokens", result))
}