use quote::{Tokens, ToTokens};
use syn::{Ident, Path};

/// This will be in scope during struct initialization after option parsing.
const DEFAULT_STRUCT_NAME: &str = "__default";

/// The fallback value for a field or container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefaultExpression<'a> {
    Inherit(&'a Ident),
    Explicit(&'a Path),
    Trait,
}

impl<'a> DefaultExpression<'a> {
    pub fn as_declaration(&'a self) -> DefaultDeclaration<'a> {
        DefaultDeclaration(self)
    }
}

impl<'a> ToTokens for DefaultExpression<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        tokens.append(match *self {
            DefaultExpression::Inherit(ident) => {
                let dsn = Ident::new(DEFAULT_STRUCT_NAME);
                quote!(#dsn.#ident)
            },
            DefaultExpression::Explicit(path) => quote!(#path()),
            DefaultExpression::Trait => quote!(::darling::export::Default::default()),
        });
    }
}

pub struct DefaultDeclaration<'a>(&'a DefaultExpression<'a>);

impl<'a> ToTokens for DefaultDeclaration<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let name = Ident::new(DEFAULT_STRUCT_NAME);
        let expr = self.0;
        tokens.append(quote!(let #name: Self = #expr;));
    }
}