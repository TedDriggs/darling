use quote::{Tokens, ToTokens};
use syn::Ident;

use codegen::{Field, TraitImpl, ExtractAttribute};

/// `impl FromField` generator. This is used for parsing an individual
/// field and its attributes.
pub struct FromFieldImpl<'a> {
    pub ident: Option<Ident>,
    pub vis: Option<Ident>,
    pub ty: Option<Ident>,
    pub body: TraitImpl<'a>,
    pub attr_names: Vec<&'a str>,
}

impl<'a> ToTokens for FromFieldImpl<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let input = quote!(__field);

        let ty_ident = self.body.ident;
        let (impl_generics, ty_generics, where_clause) = self.body.generics.split_for_impl();

        let decls = self.body.local_declarations();
        let initializers = self.body.fields.iter().map(Field::as_initializer);
        let default = self.body.fallback_decl();

        let passed_ident = self.ident.as_ref().map(|i| quote!(#i: #input.ident.clone().unwrap(),));
        let passed_vis = self.vis.as_ref().map(|i| quote!(#i: #input.vis.clone(),));
        let passed_ty = self.ty.as_ref().map(|i| quote!(#i: #input.ty.clone(),));

        /// Determine which attributes to forward (if any).
        let grab_attrs = self.extractor();

        tokens.append(quote!(
            impl #impl_generics ::darling::FromField for #ty_ident #ty_generics
                #where_clause
            {
                fn from_field(#input: &::syn::Field) -> ::darling::Result<Self> {
                    #decls

                    #grab_attrs

                    #default

                    Ok(Self {
                        #passed_ident
                        #passed_ty
                        #passed_vis
                        #(#initializers),*
                    })
                    
                }
            }
        ));
    }
}

impl<'a> ExtractAttribute for FromFieldImpl<'a> {
    fn attr_names(&self) -> &[&str] {
        self.attr_names.as_slice()
    }

    fn param_name(&self) -> Tokens {
        quote!(__field)
    }

    fn core_loop(&self) -> Tokens {
        self.body.core_loop()
    }
}