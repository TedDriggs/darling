use syn;

use {Result};
use codegen::FromTypeParamImpl;
use options::{ParseAttribute, ParseData, OuterFrom};

#[derive(Debug)]
pub struct FromTypeParamOptions {
    pub base: OuterFrom,
}

impl FromTypeParamOptions {
    pub fn new(di: &syn::DeriveInput) -> Result<Self> {
        (FromTypeParamOptions {
            base: OuterFrom::start(di),
        }).parse_attributes(&di.attrs)?.parse_body(&di.data)
    }
}

impl ParseAttribute for FromTypeParamOptions {
    fn parse_nested(&mut self, mi: &syn::Meta) -> Result<()> {
        self.base.parse_nested(mi)
    }
}

impl ParseData for FromTypeParamOptions {
    fn parse_variant(&mut self, variant: &syn::Variant) -> Result<()> {
        self.base.parse_variant(variant)
    }

    fn parse_field(&mut self, field: &syn::Field) -> Result<()> {
        self.base.parse_field(field)
    }
}

impl<'a> From<&'a FromTypeParamOptions> for FromTypeParamImpl<'a> {
    fn from(v: &'a FromTypeParamOptions) -> Self {
        FromTypeParamImpl {
            base: (&v.base.container).into(),
            ident: v.base.ident.as_ref(),
            attrs: v.base.attrs.as_ref(),
            attr_names: v.base.attr_names.as_strs(),
            forward_attrs: v.base.forward_attrs.as_ref(),
            from_ident: v.base.from_ident,
        }
    }
}
