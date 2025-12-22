use quote::ToTokens;

use crate::options::ForwardedField;
use quote::quote;

/// Creates a field literal: `field: Option<T>,`
///
/// If this ident field has `#[darling(with = ...)]`, that will be used as a function to transform
/// the `T`. If it isn't specified, then the `T` is kept as-is
pub fn create_optional(
    ident_field: &ForwardedField,
    input: &impl ToTokens,
) -> proc_macro2::TokenStream {
    create_inner(ident_field, input, true)
}

/// Creates a field literal: `field: T,`
///
/// If this ident field has `#[darling(with = ...)]`, that will be used as a function to transform
/// the `T`. If it isn't specified, then the `T` is kept as-is
pub fn create(ident_field: &ForwardedField, input: &impl ToTokens) -> proc_macro2::TokenStream {
    create_inner(ident_field, input, false)
}

fn create_inner(
    ident_field: &ForwardedField,
    input: &impl ToTokens,
    is_option_ident: bool,
) -> proc_macro2::TokenStream {
    let ident = &ident_field.ident;
    if let Some(callable) = &ident_field.with {
        let ty = &ident_field.ty;

        let input_ty = if is_option_ident {
            quote!(_darling::export::Option<_darling::export::syn::Ident>)
        } else {
            quote!(_darling::export::syn::Ident)
        };
        quote::quote! {
            #ident: _darling::export::identity::<fn(#input_ty) -> _darling::Result<#ty>>(#callable)(#input)?,
        }
    } else {
        quote::quote! {
            #ident: #input,
        }
    }
}
