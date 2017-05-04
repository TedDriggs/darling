use syn;

use ::{FromMetaItem, Error, Result};
use codegen;
use options::{Container, DefaultExpression, ParseAttribute};

lazy_static! {
    /// The default path for extracting data from a meta item. This can be overridden
    /// using the `with` attribute.
    static ref FROM_META_ITEM: syn::Path = {
        syn::parse_path("::darling::FromMetaItem::from_meta_item").unwrap()
    };
}

pub struct Field {
    pub target_name: syn::Ident,
    pub attr_name: Option<String>,
    pub ty: syn::Ty,
    pub default: Option<DefaultExpression>,
    pub with: Option<syn::Path>,
}

impl Field {
    pub fn as_codegen_field<'a>(&'a self) -> codegen::Field<'a> {
        codegen::Field {
            name_in_struct: &self.target_name,
            name_in_attr: self.attr_name.as_ref().map(|n| n.as_str()).unwrap_or(self.target_name.as_ref()),
            ty: &self.ty,
            default_expression: self.as_codegen_default(),
            with_path: self.with.as_ref().unwrap_or(&FROM_META_ITEM),
        }
    }

    fn as_codegen_default<'a>(&'a self) -> Option<codegen::DefaultExpression<'a>> {
        self.default.as_ref().map(|expr| {
            match *expr {
                DefaultExpression::Explicit(ref path) => codegen::DefaultExpression::Explicit(path),
                DefaultExpression::InheritFromStruct => codegen::DefaultExpression::InheritFromStruct(&self.target_name),
                DefaultExpression::Trait => codegen::DefaultExpression::Trait,
            }
        })
    }

    pub fn from_field(f: syn::Field, parent: Option<&Container>) -> Result<Self> {
        let target_name = f.ident.unwrap();
        let ty = f.ty;
        let base = (Field {
            target_name,
            ty,
            attr_name: None,
            default: None,
            with: None,
        }).parse_attributes(&f.attrs)?;
        
        if let Some(container) = parent {
            base.with_inherited(container)
        } else {
            Ok(base)
        }
    }

    fn with_inherited(mut self, parent: &Container) -> Result<Self> {
        if self.attr_name.is_none() {
            self.attr_name = Some(parent.rename_rule.apply_to_field(&self.target_name));
        }
        if self.default.is_none() && parent.default.is_some() {
            self.default = Some(DefaultExpression::InheritFromStruct);
        }

        Ok(self)
    }
}

impl ParseAttribute for Field {
    fn parse_nested(&mut self, mi: &syn::MetaItem) -> Result<()> {
        let name = mi.name().to_string();
        match name.as_str() {
            "rename" => { self.attr_name = FromMetaItem::from_meta_item(mi)?; Ok(()) }
            "default" => { self.default = FromMetaItem::from_meta_item(mi)?; Ok(()) }
            "with" => { self.with = Some(FromMetaItem::from_meta_item(mi)?); Ok(())}
            n => Err(Error::unknown_field(n)),
        }
    }
}