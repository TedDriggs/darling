use quote::{Tokens, ToTokens};
use syn::{MetaItem, NestedMetaItem};

use {Error, FromMetaItem, Result};
use util::Flag;

#[derive(Debug, Clone)]
pub struct Shape {
    enum_values: InnerShape,
    struct_values: InnerShape,
    any: Flag,
}

impl Shape {
    pub fn all() -> Self {
        Shape {
            any: Flag::present(),
            ..Default::default()
        }
    }
}

impl Default for Shape {
    fn default() -> Self {
        Shape {
            enum_values: InnerShape::new("enum"),
            struct_values: InnerShape::new("struct"),
            any: Default::default(),
        }
    }
}

impl FromMetaItem for Shape {
    fn from_list(items: &[NestedMetaItem]) -> Result<Self> {
        let mut new = Shape::default();
        for item in items {
            if let NestedMetaItem::MetaItem(MetaItem::Word(ref ident)) = *item {
                let word = ident.as_ref();
                if word == "any" {
                    new.any = Flag::present();
                }
                else if word.starts_with("enum_") {
                    new.enum_values.set_word(word)?;
                } else if word.starts_with("struct_") {
                    new.struct_values.set_word(word)?;
                } else {
                    return Err(Error::unknown_value(word));
                }
            } else {
                return Err(Error::unsupported_format("non-word"));
            }
        }

        Ok(new)
    }
}

impl ToTokens for Shape {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let fn_body = if self.any == true {
            quote!(::darling::export::Ok(()))
        }
        else {
            let en = &self.enum_values;
            let st = &self.struct_values;
            quote! {
                match *__body {
                    ::syn::Body::Enum(ref variants) => {
                        fn validate_variant(data: &::syn::VariantData) -> ::darling::Result<()> {
                            #en
                        }

                        for variant in variants {
                            validate_variant(&variant.data)?;
                        }

                        Ok(())
                    }
                    ::syn::Body::Struct(ref data) => {
                        #st
                    }
                }
            }
        };

        tokens.append(quote!{
            #[allow(unused_variables)]
            fn __validate_body(__body: &::syn::Body) -> ::darling::Result<()> {
                #fn_body
            }
        });
    }
}

#[derive(Debug, Clone, Default)]
struct InnerShape {
    prefix: &'static str,
    newtype: Flag,
    named: Flag,
    tuple: Flag,
    unit: Flag,
    any: Flag,
}

impl InnerShape {
    fn new(prefix: &'static str) -> Self {
        InnerShape {
            prefix,
            ..Default::default()
        }
    }

    fn supports_none(&self) -> bool {
        (self.newtype | self.named | self.tuple | self.unit | self.any).is_none()
    } 

    fn set_word(&mut self, word: &str) -> Result<()> {
        match word.trim_left_matches(self.prefix) {
            "_newtype" => {
                self.newtype = Flag::present();
                Ok(())
            }
            "_named" => {
                self.named = Flag::present();
                Ok(())
            }
            "_tuple" => {
                self.tuple = Flag::present();
                Ok(())
            }
            "_unit" => {
                self.unit = Flag::present();
                Ok(())
            }
            "_any" => {
                self.any = Flag::present();
                Ok(())
            }
            other => Err(Error::unknown_value(other)),
        }
    }
}

impl ToTokens for InnerShape {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let body = if self.any == true {
            quote!(::darling::export::Ok(()))
        } else if self.supports_none() {
            let ty = self.prefix;
            quote!(::darling::export::Err(::darling::Error::unsupported_format(#ty)))
        } else {
            let unit = match_arm("unit", self.unit.into());
            let newtype = match_arm("newtype", self.newtype.into());
            let named = match_arm("named", self.named.into());
            let tuple = match_arm("tuple", self.tuple.into());
            quote! {
                match *data {
                    ::syn::VariantData::Unit => #unit,
                    ::syn::VariantData::Tuple(ref fields) if fields.len() == 1 => #newtype,
                    ::syn::VariantData::Tuple(_) => #tuple,
                    ::syn::VariantData::Struct(_) => #named,
                }
            }
        };

        tokens.append(body);
    }
}

fn match_arm(name: &'static str, is_supported: bool) -> Tokens {
    if is_supported {
        quote!(::darling::export::Ok(()))
    } else {
        quote!(::darling::export::Err(::darling::Error::unsupported_format(#name)))
    }
}

#[cfg(test)]
mod tests {
    use syn;
    
    use super::Shape;
    use {FromMetaItem};
    use util::Flag;

    /// parse a string as a syn::MetaItem instance.
    fn pmi(s: &str) -> ::std::result::Result<syn::MetaItem, String> {
        Ok(syn::parse_outer_attr(&format!("#[{}]", s))?.value)
    }

    fn fmi<T: FromMetaItem>(s: &str) -> T {
        FromMetaItem::from_meta_item(&pmi(s).expect("Tests should pass well-formed input"))
            .expect("Tests should pass valid input")
    }

    #[test]
    fn supports_any() {
        let decl = fmi::<Shape>("ignore(any)");
        assert_eq!(decl.any, Flag::present());
    }

    #[test]
    fn supports_struct() {
        let decl = fmi::<Shape>("ignore(struct_any, struct_newtype)");
        assert_eq!(decl.struct_values.any, Flag::present());
        assert_eq!(decl.struct_values.newtype, Flag::present());
    }

    #[test]
    fn supports_mixed() {
        let decl = fmi::<Shape>("ignore(struct_newtype, enum_newtype, enum_tuple)");
        assert_eq!(decl.struct_values.newtype, Flag::present());
        assert_eq!(decl.enum_values.newtype, Flag::present());
        assert_eq!(decl.enum_values.tuple, Flag::present());
        assert_eq!(decl.struct_values.any, None);
    }
}