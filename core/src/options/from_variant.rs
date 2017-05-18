use syn::{DeriveInput, Field, Ident, MetaItem};

use Result;
use codegen::FromVariantImpl;
use options::{OuterFrom, ParseAttribute, ParseBody};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FromVariantOptions {
    pub base: OuterFrom,
    pub data: Option<Ident>,   
}

impl FromVariantOptions {
    pub fn new(di: &DeriveInput) -> Result<Self> {
        (FromVariantOptions {
            base: OuterFrom::start(di),
            data: Default::default()
        }).parse_attributes(&di.attrs)?.parse_body(&di.body)
    }
}

impl<'a> From<&'a FromVariantOptions> for FromVariantImpl<'a> {
    fn from(v: &'a FromVariantOptions) -> Self {
        FromVariantImpl {
            base: (&v.base.container).into(),
            ident: v.base.ident.as_ref(),
            data: v.data.as_ref(),
            attrs: v.base.attrs.as_ref(),
            attr_names: v.base.attr_names.as_strs(),
            forward_attrs: v.base.forward_attrs.as_ref(),
            from_ident: Some(v.base.from_ident),
        }
    }
}

impl ParseAttribute for FromVariantOptions {
    fn parse_nested(&mut self, mi: &MetaItem) -> Result<()> {
        self.base.parse_nested(mi)
    }
}

impl ParseBody for FromVariantOptions {
    fn parse_field(&mut self, field: &Field) -> Result<()> {
        match field.ident.as_ref().map(|i| i.as_ref()) {
            Some("data") => { self.data = field.ident.clone(); Ok(()) }
            _ => self.base.parse_field(field)
        }
    }
}