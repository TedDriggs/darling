use syn::{Attribute, Generics, Ident, MetaItem};

use {FromMetaItem, Result};
use codegen::FromFieldImpl;
use options::{Container, ParseAttribute};
use util::IdentList;

pub struct FromFieldOptions {
    pub ident: bool,
    pub vis: bool,
    pub ty: bool,
    pub attr_names: IdentList,
    pub container: Container,
}

impl FromFieldOptions {
    /// Create a new instance for the target name and generics.
    pub fn new(target_name: Ident, generics: Generics, attrs: &[Attribute]) -> Result<Self> {
        (FromFieldOptions {
            ident: false,
            vis: false,
            ty: false,
            attr_names: Default::default(),
            container: {
                let mut c = Container::from(target_name);
                c.generics = generics;
                c
            },
        }).parse_attributes(attrs)
    }
}

impl ParseAttribute for FromFieldOptions {
    fn parse_nested(&mut self, mi: &MetaItem) -> Result<()> {
        match mi.name() {
            "attributes" => { self.attr_names = FromMetaItem::from_meta_item(mi)?; Ok(()) }
            _ => self.container.parse_nested(mi)
        }
    }
}

impl<'a> From<&'a FromFieldOptions> for FromFieldImpl<'a> {
    fn from(v: &'a FromFieldOptions) -> Self {
        FromFieldImpl {
            ident: if v.ident { Some(Ident::new("ident")) } else { None },
            vis: if v.vis { Some(Ident::new("vis")) } else { None },
            ty: if v.ty { Some(Ident::new("ty")) } else { None },
            body: (&v.container).into(),
            attr_names: v.attr_names.as_strs(),
        }
    }
}