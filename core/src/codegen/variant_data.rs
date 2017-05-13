use quote::Tokens;

use codegen::field;
use codegen::Field;
use util::VariantData;

#[allow(dead_code)]
pub struct VariantDataGen<'a>(pub &'a VariantData<Field<'a>>);

#[allow(dead_code)]
impl<'a> VariantDataGen<'a> {
    pub fn declarations(&self) -> Tokens {
        match *self.0 {
            VariantData::Struct(ref fields) => {
                let vdr = fields.into_iter().map(Field::as_declaration);
                quote!(#(#vdr)*)
            }
            _ => panic!("VariantDataGen doesn't support tuples yet"),
        }
    }

    /// Generate the loop which walks meta items looking for property matches.
    /// TODO: Mark this as `pub(in codegen)` once restricted visibility stabilizes.
    pub fn core_loop(&self) -> Tokens {
        let arms: Vec<field::MatchArm> = self.0.as_ref().map(Field::as_match).into();

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

    pub fn initializers(&self) -> Tokens {
        let inits: Vec<_> = self.0.as_ref().map(Field::as_initializer).into();

        quote!(#(#inits),*)
    }
}