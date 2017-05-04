use serde_case::RenameRule;
use syn::{MetaItem, Ident, Generics, Attribute};

use {Result, IdentList, FromMetaItem};
use codegen;
use options::{Container, ParseAttribute};

#[derive(Debug)]
pub struct FromDeriveInputContainer {
    pub container: Container,

    /// The attribute names that should be searched.
    pub attr_names: IdentList,

    /// Whether or not the target struct has an `ident` field.
    pub ident: bool,

    /// Whether or not the target struct has a `vis` field.
    pub vis: bool,

    /// Whether or not the target struct has a `generics` field.
    pub generics: bool,
}

impl FromDeriveInputContainer {
    pub fn new(ident: Ident, generics: Generics, attrs: &[Attribute]) -> Result<Self> {
        (FromDeriveInputContainer {
            container: Container {
                ident: ident,
                generics: generics,
                default: None,
                rename_rule: RenameRule::None,
            },
            attr_names: Default::default(),
            ident: false,
            vis: false,
            generics: false,
        }).parse_attributes(attrs)
    }
}

impl<'a> From<&'a FromDeriveInputContainer> for codegen::FromDeriveInputImpl<'a> {
    fn from(v: &'a FromDeriveInputContainer) -> Self {
        codegen::FromDeriveInputImpl {
            struct_impl: (&v.container).into(),
            attr_names: v.attr_names.iter().map(|i| i.as_ref()).collect(),
            ident: if v.ident { Some(Ident::new("ident")) } else { None },
            vis: if v.vis { Some(Ident::new("vis")) } else { None },
            generics: if v.generics { Some(Ident::new("generics")) } else { None },
        }
    }
}

impl ParseAttribute for FromDeriveInputContainer {
    fn parse_nested(&mut self, mi: &MetaItem) -> Result<()> {
        match mi.name() {
            "attributes" => { self.attr_names = FromMetaItem::from_meta_item(mi)?; Ok(()) },
            "ident" => { self.ident = true; Ok(()) }
            "vis" => { self.vis = true; Ok(()) }
            "generics" => { self.generics = true; Ok(()) }
            _ => { self.container.parse_nested(mi) }
        }
    }
}