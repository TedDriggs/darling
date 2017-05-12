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

    pub fn as_ref<'a>(&'a self) -> Body<&'a V, &'a F> {
        match *self {
            Body::Enum(ref variants) => Body::Enum(variants.into_iter().collect()),
            Body::Struct(ref data) => Body::Struct(data.as_ref()),
        }
    }

    pub fn map_enum_variants<T, U>(self, map: T) -> Body<U, F> where T: FnMut(V) -> U {
        match self {
            Body::Enum(v) => Body::Enum(v.into_iter().map(map).collect()),
            Body::Struct(f) => Body::Struct(f),
        }
    }

    pub fn map_struct_fields<T, U>(self, map: T) -> Body<V, U> where T: FnMut(F) -> U {
        match self {
            Body::Enum(v) => Body::Enum(v),
            Body::Struct(f) => Body::Struct(f.map(map)),
        }
    }

    pub fn map_struct<T, U>(self, mut map: T) -> Body<V, U> where T: FnMut(VariantData<F>) -> VariantData<U> {
        match self {
            Body::Enum(v) => Body::Enum(v),
            Body::Struct(f) => Body::Struct(map(f)),
        }
    }

    pub fn take_struct(self) -> Option<VariantData<F>> {
        match self {
            Body::Enum(_) => None,
            Body::Struct(f) => Some(f),
        }
    }

    pub fn is_enum(&self) -> bool {
        match *self {
            Body::Enum(_) => true,
            Body::Struct(_) => false,
        }
    }

    pub fn is_struct(&self) -> bool {
        !self.is_enum()
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