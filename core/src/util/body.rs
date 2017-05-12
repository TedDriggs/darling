use syn::{self, Variant};
use {FromField, FromVariant, Result};
use util::VariantData;

/// Contents of a type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Body<V, F> {
    /// Type is an enum.
    Enum(Vec<V>),

    /// Type is a struct.
    Struct(VariantData<F>),
}

impl<V, F> Body<V, F> {
    /// Creates an empty body of the same shape as the passed-in body.
    pub fn empty_from(src: &syn::Body) -> Self {
        match *src {
            syn::Body::Enum(_) => Body::Enum(vec![]),
            syn::Body::Struct(ref vd) => Body::Struct(VariantData::empty_from(vd)),
        }
    }
}

impl<V: FromVariant, F> Body<V, F> {
    pub fn from_variants(variants: &[Variant]) -> Result<Self> {
        let mut v = Vec::with_capacity(variants.len());
        for variant in variants {
            v.push(FromVariant::from_variant(variant)?);
        }

        Ok(Body::Enum(v))
    }
}

impl<V: FromVariant, F: FromField> Body<V, F> {
    pub fn from_body(body: &syn::Body) -> Result<Self> {
        match *body {
            syn::Body::Enum(ref variants) => Self::from_variants(variants),
            syn::Body::Struct(ref v_data) => VariantData::from(v_data).map(Body::Struct),
        }
    }
}