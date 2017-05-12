use quote::{Tokens};
use syn::{Generics, Ident, Path};

use codegen::{DefaultExpression, Field, Variant};
use codegen::field;
use util::Body;

#[derive(Debug)]
pub struct TraitImpl<'a> {
    pub ident: &'a Ident,
    pub generics: &'a Generics,
    pub fields: Vec<Field<'a>>,
    pub body: Body<Variant<'a>, Field<'a>>,
    pub default: Option<DefaultExpression<'a>>,
    pub map: Option<&'a Path>,
}

impl<'a> TraitImpl<'a> {
    /// Generate local variable declarations for all fields.
    /// TODO: Mark this as `pub(in codegen)` once restricted visibility stabilizes.
    pub fn local_declarations(&self) -> Tokens {
        if let Body::Struct(ref vd) = self.body {
            let vdr = vd.as_ref().map(Field::as_declaration);
            let decls = vdr.fields();
            quote!(#(#decls)*)
        } else {
            quote!()
        }
    }

    /// Generate immutable variable declarations for all fields.
    /// TODO: Mark this as `pub(in codegen)` once restricted visiblity stabilizes.
    pub fn immutable_declarations(&self) -> Tokens {
        if let Body::Struct(ref vd) = self.body {
            let vdr = vd.as_ref().map(|f| field::Declaration::new(f, false));
            let decls = vdr.fields();
            quote!(#(#decls)*)
        } else {
            quote!()
        }
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