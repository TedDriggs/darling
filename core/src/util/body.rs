use syn::{Field, Variant};
use {FromField, FromVariant, Result};

/// Contents of a type.
pub enum Body<V, F> {
    Variants(Vec<V>),
    Fields(Vec<F>),
}

impl<V, F: FromField> Body<V, F> {
    pub fn from_fields(fields: &[Field]) -> Result<Self> {
        let mut f = Vec::with_capacity(fields.len());
        for field in fields {
            f.push(FromField::from_field(field)?);
        }

        Ok(Body::Fields(f))
    }
}

impl<V: FromVariant, F> Body<V, F> {
    pub fn from_variants(variants: &[Variant]) -> Result<Self> {
        let mut v = Vec::with_capacity(variants.len());
        for variant in variants {
            v.push(FromVariant::from_variant(variant)?);
        }

        Ok(Body::Variants(v))
    }
}