use quote::{Tokens, ToTokens};
use syn::Ident;

/// An enum variant.
pub struct Variant<'a> {
    pub name_in_attr: &'a str,
    pub variant_ident: &'a Ident,
    pub ty_ident: &'a Ident,
}

impl<'a> Variant<'a> {
    pub fn as_match_arm(&'a self) -> MatchArm<'a> {
        MatchArm(self)
    }
}

pub struct MatchArm<'a>(&'a Variant<'a>);

impl<'a> ToTokens for MatchArm<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let name_in_attr = self.0.name_in_attr;
        let variant_ident = self.0.variant_ident;
        let ty_ident = self.0.ty_ident;

        tokens.append(quote!(
            #name_in_attr => ::darling::export::Ok(#ty_ident::#variant_ident),
        ));
    }
}