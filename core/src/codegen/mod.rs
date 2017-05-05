use syn;
use quote::{Tokens, ToTokens};

/// This will be in scope during struct initialization after option parsing.
const DEFAULT_STRUCT_NAME: &str = "__default";

mod enum_impl;
mod field;
mod from_derive_impl;
mod from_field;
mod trait_impl;
mod variant;

pub use self::enum_impl::EnumImpl;
pub use self::field::Field;
pub use self::from_derive_impl::FromDeriveInputImpl;
pub use self::from_field::FromFieldImpl;
pub use self::trait_impl::TraitImpl;
pub use self::variant::Variant;

pub enum ImplBlock<'a> {
    Struct(TraitImpl<'a>),
    Enum(EnumImpl<'a>)
}

impl<'a> ToTokens for ImplBlock<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        match *self {
            ImplBlock::Struct(ref i) => i.to_tokens(tokens),
            ImplBlock::Enum(ref i) => i.to_tokens(tokens),
        }
    }
}

/// The fallback value for a field or container.
#[derive(Debug, Clone)]
pub enum DefaultExpression<'a> {
    Inherit(&'a syn::Ident),
    Explicit(&'a syn::Path),
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
                let dsn = syn::Ident::new(DEFAULT_STRUCT_NAME);
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
        let name = syn::Ident::new(DEFAULT_STRUCT_NAME);
        let expr = self.0;
        tokens.append(quote!(let #name: Self = #expr;));
    }
}

trait ExtractAttribute {
    fn attr_names(&self) -> &[&str];

    fn param_name(&self) -> Tokens;

    fn core_loop(&self) -> Tokens;

    fn extractor(&self) -> Tokens {
        if !self.attr_names().is_empty() {
            let input = self.param_name();
            let attr_names = self.attr_names();
            let core_loop = self.core_loop();

            quote!(
                for __attr in &#input.attrs {
                    // Filter attributes based on name
                    match __attr.name() {
                        #(#attr_names)|* => {
                            if let ::syn::MetaItem::List(_, ref __items) = __attr.value {
                                #core_loop
                            } else {
                                // darling currently only supports list-style
                                continue
                            }
                            }
                        _ => continue
                    }
                })
        } else {
            quote!()
        }
    }
}