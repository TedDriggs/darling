use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;

/// Wraps all output inside of an anonymous `const` block
///
/// Because trait implementations are hoisted to the top-level, it is possible to
/// create them inside of the `const` block and take effect after
///
/// This is done to support users defining a custom location for the `darling` crate
///
/// If not wrapped in a `const` block, then multiple uses of `derive` on `darling`'s
/// macros will create errors, because there may be multiple `extern crate darling as _darling`
///
/// # Arguments
///
/// - `tokens`: The trait implementations to wrap inside of `const` block
/// - `krate`: Path to the darling crate, which defaults to `darling`
pub fn wrap_in_const<T: ToTokens>(tokens: &T, krate: Option<&syn::Path>) -> TokenStream {
    let use_darling = krate.map_or_else(
        || quote! { use ::darling as _darling; },
        |krate| quote_spanned! { krate.span() => use #krate as _darling; },
    );

    quote! {
        #[doc(hidden)]
        const _: () = {
            #use_darling

            #tokens
        };
    }
}
