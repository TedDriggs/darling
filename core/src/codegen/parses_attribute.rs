use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::util::PathList;

use super::{OuterFromImpl, TraitImpl};

pub struct ParsesAttributeImpl<'a> {
    pub base: TraitImpl<'a>,
    pub attr_names: &'a PathList,
}

impl ToTokens for ParsesAttributeImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.attr_names.is_empty() {
            self.wrap(
                quote! {
                    fn parses(_attr: &::syn::Attribute) -> bool {
                        false
                    }
                },
                tokens,
            );

            return;
        }

        let body = if self.attr_names.is_empty() {
            quote! {
                fn parses(_attr: &::syn::Attribute) -> bool {
                    false
                }
            }
        }
        // The common case for derive macros would be a simple identifier as the attribute
        // path; in that case, converting the path to a string segment-by-segment is unnecessary.
        else if self
            .attr_names
            .iter()
            .all(|path| path.get_ident().is_some())
        {
            let ident_attr_exprs = self
                .attr_names
                .iter()
                .map(|ident| {
                    let text = ident.get_ident().expect("All paths are idents").to_string();
                    quote! {
                        attr.path.is_ident(#text)
                    }
                })
                .collect::<Vec<_>>();

            quote! {
                fn parses(attr: &::syn::Attribute) -> bool {
                    #(#ident_attr_exprs)||*
                }
            }
        } else {
            let paths = self.attr_names.to_strings();

            quote! {
                fn parses(attr: &::syn::Attribute) -> bool {
                    let path = attr.path.segments.iter()
                        .map(|s| s.ident.to_string())
                        .collect::<::darling::export::Vec<_>>()
                        .join("::");

                    #(path == #paths)||*
                }
            }
        };

        self.wrap(body, tokens);
    }
}

impl<'a> OuterFromImpl<'a> for ParsesAttributeImpl<'a> {
    fn trait_path(&self) -> syn::Path {
        path!(::darling::ParsesAttribute)
    }

    fn base(&'a self) -> &'a TraitImpl<'a> {
        &self.base
    }
}
