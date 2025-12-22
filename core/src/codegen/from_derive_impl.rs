use std::borrow::Cow;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{parse_quote, parse_quote_spanned, spanned::Spanned, Ident};

use crate::{
    ast::Data,
    codegen::{ident_field, ExtractAttribute, OuterFromImpl, TraitImpl},
    options::{DeriveInputShapeSet, ForwardedField},
    util::PathList,
};

use super::ForwardAttrs;

pub struct FromDeriveInputImpl<'a> {
    pub ident: Option<&'a ForwardedField>,
    pub generics: Option<&'a ForwardedField>,
    pub vis: Option<&'a Ident>,
    pub data: Option<&'a ForwardedField>,
    pub base: TraitImpl<'a>,
    pub attr_names: &'a PathList,
    pub forward_attrs: ForwardAttrs<'a>,
    pub from_ident: bool,
    pub supports: Option<&'a DeriveInputShapeSet>,
}

impl ToTokens for FromDeriveInputImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ty_ident = self.base.ident;
        let input = self.param_name();
        let post_transform = self.base.post_transform_call();

        if let Data::Struct(ref data) = self.base.data {
            if data.is_newtype() {
                self.wrap(
                    quote!{
                        fn from_derive_input(#input: &_darling::export::syn::DeriveInput) -> _darling::Result<Self> {
                            _darling::export::Ok(
                                #ty_ident(_darling::FromDeriveInput::from_derive_input(#input)?)
                            ) #post_transform
                        }
                    },
                    tokens,
                );

                return;
            }
        }

        let passed_ident = self
            .ident
            .as_ref()
            .map(|i| ident_field::create(i, &quote!(#input.ident.clone())));
        let passed_vis = self.vis.as_ref().map(|i| quote!(#i: #input.vis.clone(),));
        let passed_attrs = self.forward_attrs.as_initializer();

        let read_generics = self.generics.map(|generics| {
            let ident = &generics.ident;
            let with = generics
                .with
                .as_ref()
                .map(Cow::Borrowed)
                .unwrap_or_else(|| {
                    Cow::Owned(
                        parse_quote_spanned!(generics.ty.span()=> _darling::FromGenerics::from_generics),
                    )
                });

            // Note: This whole call has to be spanned, since setting the span on the `with` alone is not
            // sufficient to get rustc to point to the `with` path or magic field type in case of an error.`
            quote_spanned! {with.span()=>
                let #ident = __errors.handle(#with(&#input.generics));
            }
        });

        let pass_generics_to_receiver = self.generics.map(|g| g.as_initializer());

        let check_shape = self
            .supports
            .map(|s| s.validator_path())
            .unwrap_or_else(|| parse_quote!(_darling::export::Ok));

        let read_data = self
            .data
            .as_ref()
            .map(|i| match &i.with {
                Some(p) => p.clone(),
                None => parse_quote_spanned!(i.ty.span()=> _darling::export::TryFrom::try_from),
            })
            .unwrap_or_else(|| parse_quote!(_darling::export::Ok));

        let supports = self.supports;
        let validate_and_read_data = {
            // If the caller wants `data` read into a field, we can use `data` as the local variable name
            // because we know there are no other fields of that name.
            let let_binding = self.data.map(|d| {
                let ident = &d.ident;
                quote!(let #ident = )
            });
            quote! {
                #supports
                #let_binding __errors.handle(#check_shape(&#input.data).and_then(#read_data));
            }
        };

        let pass_data_to_receiver = self.data.map(|f| f.as_initializer());

        let inits = self.base.initializers();
        let default = if self.from_ident {
            quote!(let __default: Self = _darling::export::From::from(#input.ident.clone());)
        } else {
            self.base.fallback_decl()
        };

        let grab_attrs = self.extractor();

        let declare_errors = self.base.declare_errors();
        let require_fields = self.base.require_fields();
        let check_errors = self.base.check_errors();

        self.wrap(
            quote! {
                fn from_derive_input(#input: &_darling::export::syn::DeriveInput) -> _darling::Result<Self> {
                    #declare_errors

                    #grab_attrs

                    #validate_and_read_data

                    #read_generics

                    #require_fields

                    #check_errors

                    #default

                    _darling::export::Ok(#ty_ident {
                        #passed_ident
                        #pass_generics_to_receiver
                        #passed_vis
                        #passed_attrs
                        #pass_data_to_receiver
                        #inits
                    }) #post_transform
                }
            },
            tokens,
        );
    }
}

impl ExtractAttribute for FromDeriveInputImpl<'_> {
    fn attr_names(&self) -> &PathList {
        self.attr_names
    }

    fn forward_attrs(&self) -> &ForwardAttrs<'_> {
        &self.forward_attrs
    }

    fn param_name(&self) -> TokenStream {
        quote!(__di)
    }

    fn core_loop(&self) -> TokenStream {
        self.base.core_loop()
    }

    fn local_declarations(&self) -> TokenStream {
        self.base.local_declarations()
    }
}

impl<'a> OuterFromImpl<'a> for FromDeriveInputImpl<'a> {
    fn trait_path(&self) -> syn::Path {
        path!(_darling::FromDeriveInput)
    }

    fn trait_bound(&self) -> syn::Path {
        path!(_darling::FromMeta)
    }

    fn base(&'a self) -> &'a TraitImpl<'a> {
        &self.base
    }
}
