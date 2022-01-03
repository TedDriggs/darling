use quote::ToTokens;

use crate::{codegen::ParsesAttributeImpl, Result};

use super::{OuterFrom, ParseAttribute};

#[derive(Debug)]
pub struct ParsesAttributeOptions {
    base: OuterFrom,
}

impl ParsesAttributeOptions {
    pub fn new(di: &syn::DeriveInput) -> Result<Self> {
        (Self {
            base: OuterFrom::start(di)?,
        })
        .parse_attributes(&di.attrs)
    }
}

impl ParseAttribute for ParsesAttributeOptions {
    fn parse_nested(&mut self, mi: &syn::Meta) -> crate::Result<()> {
        self.base.parse_nested_ignore_unknown(mi).map(|_| ())
    }
}

impl<'a> From<&'a ParsesAttributeOptions> for ParsesAttributeImpl<'a> {
    fn from(v: &'a ParsesAttributeOptions) -> Self {
        Self {
            base: (&v.base.container).into(),
            attr_names: &v.base.attr_names,
        }
    }
}

impl ToTokens for ParsesAttributeOptions {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        ParsesAttributeImpl::from(self).to_tokens(tokens);
    }
}
