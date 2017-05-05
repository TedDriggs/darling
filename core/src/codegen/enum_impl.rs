use quote::{Tokens, ToTokens};
use syn::{Ident, Generics};

use codegen::Variant;

/// Data needed to generate an impl of `FromMetaItem` for a unit enum.
pub struct EnumImpl<'a> {
    /// The type ident of the target enum.
    pub ident: &'a Ident,

    /// Any generics for the enum (should be none at the moment, as only unit enums are supported).
    pub generics: &'a Generics,

    /// The variants of the enum.
    pub variants: Vec<Variant<'a>>,
}

impl<'a> ToTokens for EnumImpl<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let ident = self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let arms = self.variants.iter().map(Variant::as_match_arm);

        tokens.append(quote!(
            impl #impl_generics ::darling::FromMetaItem for #ident #ty_generics
                #where_clause 
            {
                fn from_string(lit: &str) -> ::darling::Result<Self> {
                    match lit {
                        #(#arms)*
                        __other => ::darling::export::Err(::darling::Error::unknown_value(__other))
                    }
                }
            }
        ));
    }
}