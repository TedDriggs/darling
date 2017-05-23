use syn::{MetaItem, NestedMetaItem};

use {Error, FromMetaItem, Result};
use util::Flag;

#[derive(Debug, Clone, Default)]
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

#[derive(Debug, Clone, Default)]
struct InnerShape {
    newtype: Flag,
    named: Flag,
    tuple: Flag,
    unit: Flag,
    any: Flag,
}

impl InnerShape {
    fn set_word(&mut self, word: &str) -> Result<()> {
        match word.trim_left_matches("enum_").trim_left_matches("struct_") {
            "newtype" => {
                self.newtype = Flag::present();
                Ok(())
            }
            "named" => {
                self.named = Flag::present();
                Ok(())
            }
            "tuple" => {
                self.tuple = Flag::present();
                Ok(())
            }
            "unit" => {
                self.unit = Flag::present();
                Ok(())
            }
            "any" => {
                self.any = Flag::present();
                Ok(())
            }
            other => Err(Error::unknown_value(other)),
        }
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