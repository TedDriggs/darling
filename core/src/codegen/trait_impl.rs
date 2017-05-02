use quote::{Tokens, ToTokens};
use syn::{Generics, Ident};

use codegen::{DefaultExpression, DEFAULT_STRUCT_NAME, Field};

pub struct TraitImpl<'a> {
    pub ident: &'a Ident,
    pub generics: &'a Generics,
    pub fields: Vec<Field<'a>>,
    pub default: Option<DefaultExpression<'a>>,
}

impl<'a> ToTokens for TraitImpl<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let ty_ident = self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let decls = self.fields.iter().map(Field::as_var);
        let arms = self.fields.iter().map(Field::as_match);
        let inits = self.fields.iter().map(Field::as_initializer);

        let default = self.default.as_ref().map(|p| {
            let name = DEFAULT_STRUCT_NAME;
            quote!(let #name = #p;)
        });

        tokens.append(quote!(
            impl #impl_generics ::darling::FromMetaItem for #ty_ident #ty_generics 
                #where_clause {
                fn from_list(__items: Vec<syn::NestedMetaItem>) -> ::darling::Result<Self> {
                    #(#decls)*

                    for __item in __items {
                        if let syn::NestedMetaItem::MetaItem(__inner) = __item {
                            // TODO figure out how to avoid complaints of moving after borrow here.
                            let __name = __inner.name().to_string();
                            match __name.as_str() {
                                #(#arms)*
                                __other => return ::darling::export::Err(::darling::Error::unknown_field(__other))
                            }
                        }
                    }

                    #default

                    Ok(#ty_ident {
                        #(#inits),*
                    })
                }
            }
        ));
    }
}