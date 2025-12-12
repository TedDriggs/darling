use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::{parse_quote, spanned::Spanned, FieldValue};

use crate::options::ForwardedField;
use quote::quote;

/// Creates a field literal: `field: Option<T>,`
///
/// If this ident field has `#[darling(with = ...)]`, that will be used as a function to transform
/// the `T`. If it isn't specified, then the `T` is kept as-is
pub fn create_optional(ident_field: &ForwardedField, input: &TokenStream) -> FieldValue {
    create_inner(ident_field, input, true)
}

/// Creates a field literal: `field: T,`
///
/// If this ident field has `#[darling(with = ...)]`, that will be used as a function to transform
/// the `T`. If it isn't specified, then the `T` is kept as-is
pub fn create(ident_field: &ForwardedField, input: &TokenStream) -> FieldValue {
    create_inner(ident_field, input, false)
}

fn create_inner(
    ident_field: &ForwardedField,
    input: &TokenStream,
    is_option_ident: bool,
) -> FieldValue {
    let ident = &ident_field.ident;

    // If the error has a type mismatch, point to the identifier in the field.
    //
    // Adding parentheses around the expression `#input.ident` will make the error
    // point to field name of the user's type. Without them, the error is at `Span::call_site()`
    let input = quote_spanned! {
        ident_field.ident.span() => _darling::export::Clone::clone(&(#input.ident))
    };

    if let Some(callable) = &ident_field.with {
        let ty = &ident_field.ty;

        let input_ty = if is_option_ident {
            quote!(_darling::export::Option<_darling::export::syn::Ident>)
        } else {
            quote!(_darling::export::syn::Ident)
        };
        let value = quote_spanned! { callable.span() =>
            _darling::export::identity::<fn(#input_ty) -> _darling::Result<#ty>>(#callable)(#input)?
        };
        parse_quote! {
            #ident: #value
        }
    } else {
        parse_quote! {
            #ident: #input
        }
    }
}
