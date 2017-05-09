use quote::{Tokens, ToTokens};
use syn::Ident;

use codegen::{Field, TraitImpl, ExtractAttribute};
use options::ForwardAttrs;

pub struct FromDeriveInputImpl<'a> {
    pub ident: Option<&'a Ident>,
    pub generics: Option<&'a Ident>,
    pub vis: Option<&'a Ident>,
    pub attrs: Option<&'a Ident>,
    pub struct_impl: TraitImpl<'a>,
    pub attr_names: Vec<&'a str>,
    pub forward_attrs: Option<&'a ForwardAttrs>,
    pub from_ident: Option<bool>,
}

impl<'a> ToTokens for FromDeriveInputImpl<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let input = self.param_name();
        
        let passed_ident = self.ident.as_ref().map(|i| quote!(#i: #input.ident.clone(),));
        let passed_vis = self.vis.as_ref().map(|i| quote!(#i: #input.vis.clone(),));
        let passed_generics = self.generics.as_ref().map(|i| quote!(#i: #input.generics.clone(),));
        let passed_attrs = self.attrs.as_ref().map(|i| quote!(#i: __fwd_attrs,));

        let ty_ident = self.struct_impl.ident;
        let (impl_generics, ty_generics, where_clause) = self.struct_impl.generics.split_for_impl();
        let inits = self.struct_impl.fields.iter().map(Field::as_initializer);
        let default = if let Some(true) = self.from_ident {
            quote!(let __default: Self = ::darling::export::From::from(#input.ident.clone());)
        } else {
            self.struct_impl.fallback_decl()
        };

        let grab_attrs = self.extractor();
        let map = self.struct_impl.map_fn();

        tokens.append(quote!(
            impl #impl_generics ::darling::FromDeriveInput for #ty_ident #ty_generics
                #where_clause
            {
                fn from_derive_input(#input: &::syn::DeriveInput) -> ::darling::Result<Self> {
                    #grab_attrs

                    #default

                    Ok(#ty_ident {
                        #passed_ident
                        #passed_generics
                        #passed_vis
                        #passed_attrs
                        #(#inits),*
                    }) #map
                }
            } 
        ));
    }
}

impl<'a> ExtractAttribute for FromDeriveInputImpl<'a> {
    fn attr_names(&self) -> &[&str] {
        self.attr_names.as_slice()
    }

    fn forwarded_attrs(&self) -> Option<&ForwardAttrs> {
        self.forward_attrs
    }

    fn param_name(&self) -> Tokens {
        quote!(__di)
    }

    fn core_loop(&self) -> Tokens {
        self.struct_impl.core_loop()
    }

    fn local_declarations(&self) -> Tokens {
        self.struct_impl.local_declarations()
    }

    fn immutable_declarations(&self) -> Tokens {
        self.struct_impl.immutable_declarations()
    }
}