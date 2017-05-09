use quote::{Tokens, ToTokens};
use syn::{Generics, Ident, Path};

use codegen::{DefaultExpression, Field};
use codegen::field;

#[derive(Debug)]
pub struct TraitImpl<'a> {
    pub ident: &'a Ident,
    pub generics: &'a Generics,
    pub fields: Vec<Field<'a>>,
    pub default: Option<DefaultExpression<'a>>,
    pub include_applicator: bool,
    pub map: Option<&'a Path>,
}

impl<'a> TraitImpl<'a> {
    /// Generate local variable declarations for all fields.
    /// TODO: Mark this as `pub(in codegen)` once restricted visibility stabilizes.
    pub fn local_declarations(&self) -> Tokens {
        let decls = self.fields.iter().map(Field::as_declaration);
        quote!(#(#decls)*)
    }

    /// Generate immutable variable declarations for all fields.
    /// TODO: Mark this as `pub(in codegen)` once restricted visiblity stabilizes.
    pub fn immutable_declarations(&self) -> Tokens {
        let decls = self.fields
            .iter()
            .map(|f| field::Declaration::new(f, false));
        quote!(#(#decls)*)
    }

    pub fn map_fn(&self) -> Option<Tokens> {
        self.map.as_ref().map(|path| quote!(.map(#path)))
    }

    /// Generate local variable declaration and initialization for instance from which missing fields will be taken.
    /// TODO: Mark this as `pub(in codegen)` once restricted visibility stabilizes.
    pub fn fallback_decl(&self) -> Tokens {
        let default = self.default.as_ref().map(DefaultExpression::as_declaration);
        quote!(#default)
    }

    /// Generate the loop which walks meta items looking for property matches.
    /// TODO: Mark this as `pub(in codegen)` once restricted visibility stabilizes.
    pub fn core_loop(&self) -> Tokens {
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
        let default = self.fallback_decl();
        let map = self.map_fn();
        

        tokens.append(quote!(
            impl #impl_generics ::darling::FromMetaItem for #ty_ident #ty_generics 
                #where_clause {
                fn from_list(__items: &[::syn::NestedMetaItem]) -> ::darling::Result<Self> {
                    
                    #decls

                    #core_loop

                    #default

                    Ok(#ty_ident {
                        #(#inits),*
                    }) #map
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