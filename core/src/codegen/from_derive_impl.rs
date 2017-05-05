use quote::{Tokens, ToTokens};
use syn::Ident;

use codegen::{Field, TraitImpl};
use codegen::field;

pub struct FromDeriveInputImpl<'a> {
    pub ident: Option<Ident>,
    pub generics: Option<Ident>,
    pub vis: Option<Ident>,
    pub struct_impl: TraitImpl<'a>,
    pub attr_names: Vec<&'a str>,
    pub from_ident: Option<bool>,
}

impl<'a> ToTokens for FromDeriveInputImpl<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let input = quote!(__di);
        
        let passed_ident = self.ident.as_ref().map(|i| quote!(#i: #input.ident.clone(),));
        let passed_vis = self.vis.as_ref().map(|i| quote!(#i: #input.vis.clone(),));
        let passed_generics = self.generics.as_ref().map(|i| quote!(#i: #input.generics.clone(),));

        let ty_ident = self.struct_impl.ident;
        let (impl_generics, ty_generics, where_clause) = self.struct_impl.generics.split_for_impl();
        let inits = self.struct_impl.fields.iter().map(Field::as_initializer);
        let default = if let Some(true) = self.from_ident {
            quote!(let __default: Self = ::darling::export::From::from(#input.ident.clone());)
        } else {
            self.struct_impl.fallback_decl()
        };

        let decls = if !self.attr_names.is_empty() {
            self.struct_impl.local_declarations()
        } else {
            let d = self.struct_impl
                .fields
                .iter()
                .map(|f| field::Declaration::new(f, false));
            quote!(#(#d)*)
        };

        let grab_attrs = if !self.attr_names.is_empty() {
            let attr_names = &self.attr_names;
            let core_loop = self.struct_impl.core_loop();
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
        };

        tokens.append(quote!(
            impl #impl_generics ::darling::FromDeriveInput for #ty_ident #ty_generics
                #where_clause
            {
                fn from_derive_input(#input: &::syn::DeriveInput) -> ::darling::Result<Self> {
                    #decls

                    #grab_attrs

                    #default

                    Ok(#ty_ident {
                        #passed_ident
                        #passed_generics
                        #passed_vis
                        #(#inits),*
                    })
                }
            } 
        ));
    }
}