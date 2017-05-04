use serde_case::RenameRule;
use syn;

use {Result, Error, FromMetaItem};
use codegen;
use options::{DefaultExpression, ParseAttribute};

pub struct Container {
    pub ident: syn::Ident,
    pub generics: syn::Generics,
    pub default: Option<DefaultExpression>,
    pub rename_rule: RenameRule,
}

impl Container {
    /// Creates a new container, with the identity bound for later diagnostics.
    pub fn new(ident: syn::Ident, generics: syn::Generics, attrs: &[syn::Attribute]) -> Result<Self> {
        (Container {
            ident: ident,
            generics: generics,
            default: None,
            rename_rule: RenameRule::None,
        }).parse_attributes(attrs)
    }

    fn as_codegen_default<'a>(&'a self) -> Option<codegen::DefaultExpression<'a>> {
        self.default.as_ref().map(|expr| {
            match *expr {
                DefaultExpression::Explicit(ref path) => codegen::DefaultExpression::Explicit(path),
                DefaultExpression::InheritFromStruct | 
                DefaultExpression::Trait => codegen::DefaultExpression::Trait,
            }
        })
    }
}

impl ParseAttribute for Container {
    fn parse_nested(&mut self, mi: &syn::MetaItem) -> Result<()> {
        match mi.name() {
            // DANGER: Removing this causes a stack overflow.
            // "default" => { self.default = Some(FromMetaItem::from_meta_item(mi)?); Ok(()) }
            "rename_all" => { self.rename_rule = FromMetaItem::from_meta_item(mi)?; Ok(()) },
            n => Err(Error::unknown_field(n))
        }
    }
}

impl<'a> From<&'a Container> for codegen::TraitImpl<'a> {
    fn from(v: &'a Container) -> Self {
        codegen::TraitImpl {
            ident: &v.ident,
            generics: &v.generics,
            fields: vec![],
            default: v.as_codegen_default(),
            include_applicator: true,
        }
    }
}

impl<'a> From<&'a Container> for codegen::EnumImpl<'a> {
    fn from(v: &'a Container) -> Self {
        codegen::EnumImpl {
            ident: &v.ident,
            generics: &v.generics,
            variants: vec![],
        }
    }
}