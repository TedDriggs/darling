use quote::{Tokens, ToTokens};
use syn::Ident;

use codegen::TraitImpl;
use options::ForwardAttrs;

/// Wrapper for "outer From" traits, such as `FromDeriveInput`, `FromVariant`, and `FromField`.
pub trait OuterFromImpl<'a> {
    fn trait_path(&self) -> Tokens;

    fn base(&'a self) -> &'a TraitImpl<'a>;

    fn param_name(&self) -> Ident;

    fn parse_attr_names(&'a self) -> &'a[&'a str];

    fn ident_into(&'a self) -> Option<&'a Ident> {
        None
    }

    fn generics_into(&'a self) -> Option<&'a Ident> {
        None
    }

    fn attrs_into(&'a self) -> Option<&'a Ident> {
        None
    }


    fn forward_attr_names(&'a self) -> Option<&'a ForwardAttrs> {
        None
    }

    fn wrap<T: ToTokens>(&'a self, body: T, tokens: &mut Tokens) {
        let base = self.base();
        let trayt = self.trait_path();
        let ty_ident = base.ident;
        let (impl_generics, ty_generics, where_clause) = base.generics.split_for_impl();
        
        tokens.append(quote!(
            impl #impl_generics #trayt for #ty_ident #ty_generics
                #where_clause
            {
                #body
            }
        ));
    }
}