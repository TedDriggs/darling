use crate::{Error, FromMeta, Result};
use quote::{quote, ToTokens};
use syn::Ident;

use crate::{options::ParseAttribute, util::Callable};

/// Attaches error message to `Option<Ident>`, turning it into `darling::Result<Ident>`,
/// so it composes nicely with `#[darling(with = ...)]` on a `struct` deriving [`FromField`](crate::FromField)
///
/// Using `#[derive(FromField)]` requires the `ident` field, if present, to be `Option<Ident>`.
/// This is correct if you will use this `struct` as both part of a tuple struct, and a struct with
/// named fields.
///
/// However, if you don't need to do that, then there's no need to wrap it in an `Option`.
///
/// ```ignore
/// #[derive(FromField)]
/// struct Field {
///     // Without #[darling(with)], this must be `ident: Option<syn::Ident>`
///     #[darling(with = darling::util::require_ident)]
///     ident: syn::Ident,
/// }
///
/// #[derive(FromDeriveInput)]
/// struct Input {
///     data: darling_core::ast::Data<darling_core::util::Ignored, Field>,
/// }
///
/// let input = Input::from_derive_input(&parse_quote! {
///     struct Demo<T> {
///         hello: T
///     }
/// })?;
///
/// let fields = input.data.take_struct()?;
/// let first_field = fields.into_iter().next()?;
///
/// // Not wrapped in `Some`!
/// assert_eq!(first_field.ident, Ident::new("hello", Span::call_site()));
/// ```
pub fn require_ident(ident: Option<Ident>) -> crate::Result<Ident> {
    ident.ok_or_else(|| Error::custom("expected identifier"))
}

/// Field used for an `ident: Ident`
#[derive(Debug, Clone)]
pub(crate) struct IdentField {
    /// Identifier `ident` itself
    pub ident: Ident,
    /// Type of the `ident`, only used if `with` is `Some` to specify the
    /// return type of the closure/function argument to `with`
    pub ty: syn::Type,
    /// #[darling(with = my_fn)] or #[darling(with = || closure)]
    pub with: Option<Callable>,
}

impl IdentField {
    /// Creates a field literal: `field: Option<T>,`
    ///
    /// If this ident field has `#[darling(with = ...)]`, that will be used as a function to transform
    /// the `T`. If it isn't specified, then the `T` is kept as-is
    pub fn create_field_optional(&self, input: &impl ToTokens) -> proc_macro2::TokenStream {
        self.create_field_inner(input, true)
    }

    /// Creates a field literal: `field: T,`
    ///
    /// If this ident field has `#[darling(with = ...)]`, that will be used as a function to transform
    /// the `T`. If it isn't specified, then the `T` is kept as-is
    pub fn create_field(&self, input: &impl ToTokens) -> proc_macro2::TokenStream {
        self.create_field_inner(input, false)
    }

    fn create_field_inner(
        &self,
        input: &impl ToTokens,
        is_option_ident: bool,
    ) -> proc_macro2::TokenStream {
        let ident = &self.ident;
        if let Some(callable) = &self.with {
            let ty = &self.ty;

            let input_ty = if is_option_ident {
                quote!(::darling::export::Option<::darling::export::syn::Ident>)
            } else {
                quote!(::darling::export::syn::Ident)
            };
            quote::quote! {
                #ident: ::darling::export::identity::<fn(#input_ty) -> ::darling::Result<#ty>>(#callable)(#input)?,
            }
        } else {
            quote::quote! {
                #ident: #input,
            }
        }
    }
}

impl ParseAttribute for IdentField {
    fn parse_nested(&mut self, mi: &syn::Meta) -> Result<()> {
        let path = mi.path();

        if path.is_ident("with") {
            self.with = FromMeta::from_meta(mi)?;
        } else {
            return Err(Error::unknown_field_path(path).with_span(mi));
        }

        Ok(())
    }
}
