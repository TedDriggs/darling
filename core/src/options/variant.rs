use syn;

use {FromMetaItem, Error, Result};
use codegen;
use options::{Core, ParseAttribute};

pub struct Variant {
    ident: syn::Ident,
    attr_name: Option<String>,
}

impl Variant {
    pub fn as_codegen_variant<'a>(&'a self, ty_ident: &'a syn::Ident) -> codegen::Variant<'a> {
        codegen::Variant {
            name_in_attr: self.attr_name.as_ref().map(|s| s.as_str()).unwrap_or(self.ident.as_ref()),
            variant_ident: &self.ident,
            ty_ident,
        }
    }

    pub fn from_variant(v: syn::Variant, parent: Option<&Core>) -> Result<Self> {
        let starter = (Variant {
            ident: v.ident,
            attr_name: Default::default()
        }).parse_attributes(&v.attrs)?;

        Ok(if let Some(p) = parent {
            starter.with_inherited(p)
        } else {
            starter
        })
    }

    fn with_inherited(mut self, parent: &Core) -> Self {
        if self.attr_name.is_none() {
            self.attr_name = Some(parent.rename_rule.apply_to_variant(&self.ident));
        }

        self
    }
}

impl ParseAttribute for Variant {
    fn parse_nested(&mut self, mi: &syn::MetaItem) -> Result<()> {
        let name = mi.name().to_string();
        match name.as_str() {
            "rename" => { self.attr_name = FromMetaItem::from_meta_item(mi)?; Ok(()) }
            n => Err(Error::unknown_field(n)),
        }
    }
}