use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_quote, parse_quote_spanned, Ident};

use crate::codegen::{ident_field, ExtractAttribute, ForwardAttrs, OuterFromImpl, TraitImpl};
use crate::options::{DataShape, ForwardedField};
use crate::util::PathList;

pub struct FromVariantImpl<'a> {
    pub base: TraitImpl<'a>,
    /// If set, the ident of the field into which the variant ident should be placed.
    ///
    /// This is one of `darling`'s "magic fields", which allow a type deriving a `darling`
    /// trait to get fields from the input `syn` element added to the deriving struct
    /// automatically.
    pub ident: Option<&'a ForwardedField>,
    /// If set, the ident of the field into which the transformed output of the input
    /// variant's fields should be placed.
    ///
    /// This is one of `darling`'s "magic fields".
    pub fields: Option<&'a ForwardedField>,
    /// If set, the ident of the field into which the discriminant of the input variant
    /// should be placed. The receiving field must be an `Option` as not all enums have
    /// discriminants.
    ///
    /// This is one of `darling`'s "magic fields".
    pub discriminant: Option<&'a Ident>,
    pub attr_names: &'a PathList,
    pub forward_attrs: ForwardAttrs<'a>,
    pub from_ident: bool,
    pub supports: Option<&'a DataShape>,
}

impl ToTokens for FromVariantImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let input = self.param_name();
        let extractor = self.extractor();

        let forwarded_fields = vec![
            self.ident.as_ref().map(|i| ident_field::create(i, &input)),
            self.discriminant.as_ref().map(
                |i| parse_quote!(#i: #input.discriminant.as_ref().map(|(_, expr)| expr.clone())),
            ),
            self.forward_attrs.to_field_value(),
            self.fields.as_ref().map(|i| i.to_field_value()),
        ]
        .into_iter()
        .flatten();

        let inits = self.base.initializers();
        let post_transform = self.base.post_transform_call();

        let default = if self.from_ident {
            quote!(let __default: Self = _darling::export::From::from(#input.ident.clone());)
        } else {
            self.base.fallback_decl()
        };

        let read_fields = self
            .fields
            .as_ref()
            .map(|i| match &i.with {
                Some(p) => p.clone(),
                None => parse_quote_spanned!(i.ty.span()=> _darling::ast::Fields::try_from),
            })
            .unwrap_or_else(|| parse_quote!(_darling::Result::Ok));

        let supports = self
            .supports
            .map(|i| {
                quote! {
                    #i.check
                }
            })
            .unwrap_or_else(|| quote!(_darling::export::Ok));
        let validate_and_read_fields = {
            // If the caller wants `fields` read into a field, we can use `fields` as the local variable name
            // because we know there are no other fields of that name.
            let let_binding = self.fields.map(|d| {
                let ident = &d.ident;
                quote!(let #ident = )
            });

            // The awkward `map` here is to work around the fact that `ShapeSet::check` returns `()`,
            // but we need to return the fields for further processing.
            quote! {
                #let_binding __errors.handle(#supports(&#input.fields).map(|_| &#input.fields).and_then(#read_fields));
            }
        };

        let error_declaration = self.base.declare_errors();
        let require_fields = self.base.require_fields();
        let error_check = self.base.check_errors();

        self.wrap(
            quote!(
                fn from_variant(#input: &_darling::export::syn::Variant) -> _darling::Result<Self> {
                    #error_declaration

                    #extractor

                    #validate_and_read_fields

                    #require_fields

                    #error_check

                    #default

                    _darling::export::Ok(Self {
                        #(#forwarded_fields,)*
                        #inits
                    }) #post_transform
                }
            ),
            tokens,
        );
    }
}

impl ExtractAttribute for FromVariantImpl<'_> {
    fn local_declarations(&self) -> TokenStream {
        self.base.local_declarations()
    }

    fn attr_names(&self) -> &PathList {
        self.attr_names
    }

    fn forward_attrs(&self) -> &ForwardAttrs<'_> {
        &self.forward_attrs
    }

    fn param_name(&self) -> TokenStream {
        quote!(__variant)
    }

    fn core_loop(&self) -> TokenStream {
        self.base.core_loop()
    }
}

impl<'a> OuterFromImpl<'a> for FromVariantImpl<'a> {
    fn trait_path(&self) -> syn::Path {
        path!(_darling::FromVariant)
    }

    fn trait_bound(&self) -> syn::Path {
        path!(_darling::FromMeta)
    }

    fn base(&'a self) -> &'a TraitImpl<'a> {
        &self.base
    }
}
