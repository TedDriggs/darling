use quote::{Tokens, ToTokens};
use syn::{self, Ident};

use codegen::{TraitImpl, ExtractAttribute, OuterFromImpl};
use options::ForwardAttrs;

/// `impl FromField` generator. This is used for parsing an individual
/// field and its attributes.
pub struct FromFieldImpl<'a> {
    pub ident: Option<&'a Ident>,
    pub vis: Option<&'a Ident>,
    pub ty: Option<&'a Ident>,
    pub attrs: Option<&'a Ident>,
    pub body: TraitImpl<'a>,
    pub attr_names: Vec<&'a str>,
    pub forward_attrs: Option<&'a ForwardAttrs>,
    pub from_ident: bool,
}

impl<'a> ToTokens for FromFieldImpl<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let input = self.param_name();

        let initializers = self.body.initializers();
        
        let default = if self.from_ident {
            quote!(let __default: Self = ::darling::export::From::from(#input.ident.clone());)
        } else {
            self.body.fallback_decl()
        };

        let passed_ident = self.ident.as_ref().map(|i| quote!(#i: #input.ident.clone().unwrap(),));
        let passed_vis = self.vis.as_ref().map(|i| quote!(#i: #input.vis.clone(),));
        let passed_ty = self.ty.as_ref().map(|i| quote!(#i: #input.ty.clone(),));
        let passed_attrs = self.attrs.as_ref().map(|i| quote!(#i: __fwd_attrs,));

        /// Determine which attributes to forward (if any).
        let grab_attrs = self.extractor();
        let map = self.body.map_fn();

        self.wrap(quote!{
            fn from_field(#input: &::syn::Field) -> ::darling::Result<Self> {
                #grab_attrs

                #default

                ::darling::export::Ok(Self {
                    #passed_ident
                    #passed_ty
                    #passed_vis
                    #passed_attrs
                    #initializers
                }) #map
                
            }
        }, tokens);
    }
}

impl<'a> ExtractAttribute for FromFieldImpl<'a> {
    fn attr_names(&self) -> &[&str] {
        self.attr_names.as_slice()
    }

    fn forwarded_attrs(&self) -> Option<&ForwardAttrs> {
        self.forward_attrs
    }

    fn param_name(&self) -> Tokens {
        quote!(__field)
    }

    fn core_loop(&self) -> Tokens {
        self.body.core_loop()
    }

    fn local_declarations(&self) -> Tokens {
        self.body.local_declarations()
    }

    fn immutable_declarations(&self) -> Tokens {
        self.body.immutable_declarations()
    }
}

impl<'a> OuterFromImpl<'a> for FromFieldImpl<'a> {
    fn trait_path(&self) -> syn::Path {
        syn::parse_path("::darling::FromField").unwrap()
    }

    fn trait_bound(&self) -> syn::Path {
        syn::parse_path("::darling::FromMetaItem").unwrap()
    }

    fn base(&'a self) -> &'a TraitImpl<'a> {
        &self.body
    }
}