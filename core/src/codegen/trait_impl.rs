use quote::{Tokens, ToTokens};
use syn::{Generics, Ident};

use codegen::{DefaultExpression, Field};

pub struct TraitImpl<'a> {
    pub ident: &'a Ident,
    pub generics: &'a Generics,
    pub fields: Vec<Field<'a>>,
    pub default: Option<DefaultExpression<'a>>,
    pub include_applicator: bool,
}

impl<'a> TraitImpl<'a> {
    pub fn as_from_derive_input(&'a self) -> FromDeriveInputImpl<'a> {
        FromDeriveInputImpl(self)
    }

    fn local_declarations(&self) -> Tokens {
        let decls = self.fields.iter().map(Field::as_var);
        quote!(#(#decls)*)
    }

    fn core_loop(&self) -> Tokens {
        let arms = self.fields.iter().map(Field::as_match);

        quote!(
            for __item in __items {
                if let ::syn::NestedMetaItem::MetaItem(ref __inner) = *__item {
                    let __name = __inner.name().to_string();
                    match __name.as_str() {
                        #(#arms)*
                        __other => return ::darling::export::Err(::darling::Error::unknown_field(__other))
                    }
                }
            }
        )
    }
}

impl<'a> ToTokens for TraitImpl<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let ty_ident = self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let inits = self.fields.iter().map(Field::as_initializer);
        let decls = self.local_declarations();
        let core_loop = self.core_loop();

        let default = self.default.as_ref().map(DefaultExpression::as_declaration);

        tokens.append(quote!(
            impl #impl_generics ::darling::FromMetaItem for #ty_ident #ty_generics 
                #where_clause {
                fn from_list(__items: &[::syn::NestedMetaItem]) -> ::darling::Result<Self> {
                    
                    #decls

                    #core_loop

                    #default

                    Ok(#ty_ident {
                        #(#inits),*
                    })
                }
            }
        ));

        if self.include_applicator {
            let applicators = self.fields.iter().map(Field::as_applicator);

            tokens.append(quote!(
                impl #impl_generics ::darling::ApplyMetaItem for #ty_ident #ty_generics
                    #where_clause 
                    {
                        fn from_list(&mut self, __items: &[::syn::NestedMetaItem]) -> ::darling::Result<&mut Self> {
                            
                            #decls

                            #core_loop

                            #(#applicators)*

                            Ok(self)
                        }
                    }
            ));
        }
    }
}

pub struct FromDeriveInputImpl<'a>(&'a TraitImpl<'a>);

impl<'a> ToTokens for FromDeriveInputImpl<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let attrs = "darling";
        let passed_ident = quote!(ident);
        let passed_vis = quote!(vis);
        let passed_generics = quote!(generics);

        let ty_ident = self.0.ident;
        let (impl_generics, ty_generics, where_clause) = self.0.generics.split_for_impl();
        let inits = self.0.fields.iter().map(Field::as_initializer);
        let decls = self.0.local_declarations();
        let core_loop = self.0.core_loop();
        let default = self.0.default.as_ref().map(DefaultExpression::as_declaration);

        tokens.append(quote!(
            impl #impl_generics ::darling::FromDeriveInput for #ty_ident #ty_generics
                #where_clause
            {
                fn from_derive_input(di: &::syn::DeriveInput) -> ::darling::Result<Self> {
                    let #passed_ident = di.ident.clone();
                    let #passed_vis = di.vis.clone();
                    let #passed_generics = di.generics.clone();
                    #decls

                    for __attr in &di.attrs {
                        match __attr.name() {
                            #attrs => {
                                if let ::syn::MetaItem::List(_, ref __items) = __attr.value {
                                    #core_loop
                                } else {
                                    // darling currently only supports list-style
                                    continue
                                }
                             }
                            _ => continue
                        }
                    }

                    #default

                    Ok(#ty_ident {
                        #passed_ident: #passed_ident,
                        #passed_generics: #passed_generics,
                        #passed_vis: #passed_vis,
                        #(#inits),*
                    })
                }
            } 
        ));
    }
}