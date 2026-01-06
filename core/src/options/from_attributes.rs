use quote::ToTokens;

use crate::{codegen::FromAttributesImpl, Error, Result};

use super::{OuterFrom, ParseAttribute, ParseData};

/// Receiver for derived `FromAttributes` impls.
pub struct FromAttributesOptions {
    // Note: FromAttributes has no behaviors beyond those common
    // to all the `OuterFrom` traits.
    pub base: OuterFrom,
}

impl FromAttributesOptions {
    pub fn new(di: &syn::DeriveInput) -> Result<Self> {
        let opts = (Self {
            base: OuterFrom::start(di)?,
        })
        .parse_attributes(&di.attrs)?
        .parse_body(&di.data)?;

        let is_transparent = opts
            .base
            .container
            .data
            .as_struct()
            .map(|fields| {
                (fields.len() == 1 && fields.style.is_tuple())
                    || fields.style.is_struct() && opts.base.container.transparent.is_present()
            })
            .unwrap_or(false);

        if !is_transparent && opts.base.attr_names.is_empty() {
            Err(Error::custom(
                "FromAttributes without attributes collects nothing",
            ))
        } else {
            Ok(opts)
        }
    }
}

impl ParseAttribute for FromAttributesOptions {
    fn parse_nested(&mut self, mi: &syn::Meta) -> Result<()> {
        self.base.parse_nested(mi)
    }
}

impl ParseData for FromAttributesOptions {
    fn parse_variant(&mut self, variant: &syn::Variant) -> Result<()> {
        self.base.parse_variant(variant)
    }

    fn parse_field(&mut self, field: &syn::Field) -> Result<()> {
        self.base.parse_field(field)
    }

    fn validate_body(&self, errors: &mut crate::error::Accumulator) {
        self.base.validate_body(errors);
    }
}

impl<'a> From<&'a FromAttributesOptions> for FromAttributesImpl<'a> {
    fn from(v: &'a FromAttributesOptions) -> Self {
        FromAttributesImpl {
            base: (&v.base.container).into(),
            attr_names: &v.base.attr_names,
            forward_attrs: v.base.as_forward_attrs(),
        }
    }
}

impl ToTokens for FromAttributesOptions {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        FromAttributesImpl::from(self).to_tokens(tokens)
    }
}
