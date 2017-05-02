use syn;
use quote::{Tokens, ToTokens};

/// This will be in scope during struct initialization after option parsing.
const DEFAULT_STRUCT_NAME: &str = "__default";

mod field;
mod trait_impl;

pub use self::field::Field;
pub use self::trait_impl::TraitImpl;

/// The fallback value for a field (or struct).
pub enum DefaultExpression<'a> {
    InheritFromStruct(&'a syn::Ident),
    Explicit(&'a syn::Path),
    Trait,
}

impl<'a> ToTokens for DefaultExpression<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        tokens.append(match *self {
            DefaultExpression::InheritFromStruct(ident) => {
                let dsn = DEFAULT_STRUCT_NAME;
                quote!(#dsn.#ident)
            },
            DefaultExpression::Explicit(path) => quote!(#path()),
            DefaultExpression::Trait => quote!(::attr_deserialize_export::Default::default()),
        });
    }
}