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
    
    let container = options::Container::new(ast.ident, ast.generics);

    let fields = match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(fields_in)) => {
            let mut fields = Vec::with_capacity(fields_in.len());
            for field_in in fields_in {
                fields.push(options::Field::from_field(field_in).unwrap());
            }

            fields
        },
        bd => panic!("Unsupported body `{:?}`", bd)
    };

    let trait_impl = codegen::TraitImpl {
        fields: fields.iter().map(options::Field::as_codegen_field).collect(),
        ..(&container).into()
    };

    let result = quote!(#trait_impl);

    result.parse().expect(&format!("Couldn't parse `{}` to tokens", result))
}