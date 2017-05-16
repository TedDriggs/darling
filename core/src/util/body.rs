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

    /// Creates a new `Body<&'a V, &'a F>` instance from `Body<V, F>`.
    pub fn as_ref<'a>(&'a self) -> Body<&'a V, &'a F> {
        match *self {
            Body::Enum(ref variants) => Body::Enum(variants.into_iter().collect()),
            Body::Struct(ref data) => Body::Struct(data.as_ref()),
        }
    }

    /// Applies a function `V -> U` on enum variants, if this is an enum.
    pub fn map_enum_variants<T, U>(self, map: T) -> Body<U, F>
        where T: FnMut(V) -> U
    {
        match self {
            Body::Enum(v) => Body::Enum(v.into_iter().map(map).collect()),
            Body::Struct(f) => Body::Struct(f),
        }
    }

    /// Applies a function `F -> U` on struct fields, if this is a struct.
    pub fn map_struct_fields<T, U>(self, map: T) -> Body<V, U>
        where T: FnMut(F) -> U
    {
        match self {
            Body::Enum(v) => Body::Enum(v),
            Body::Struct(f) => Body::Struct(f.map(map)),
        }
    }

    /// Applies a function to the `VariantData` if this is a struct.
    pub fn map_struct<T, U>(self, mut map: T) -> Body<V, U>
        where T: FnMut(VariantData<F>) -> VariantData<U>
    {
        match self {
            Body::Enum(v) => Body::Enum(v),
            Body::Struct(f) => Body::Struct(map(f)),
        }
    }

    /// Consumes the `Body`, returning `VariantData<F>` if it was a struct.
    pub fn take_struct(self) -> Option<VariantData<F>> {
        match self {
            Body::Enum(_) => None,
            Body::Struct(f) => Some(f),
        }
    }

    /// Consumes the `Body`, returning `Vec<V>` if it was an enum.
    pub fn take_enum(self) -> Option<Vec<V>> {
        match self {
            Body::Enum(v) => Some(v),
            Body::Struct(_) => None,
        }
    }

    /// Returns `true` if this instance is `Body::Enum`.
    pub fn is_enum(&self) -> bool {
        match *self {
            Body::Enum(_) => true,
            Body::Struct(_) => false,
        }
    }

    /// Returns `true` if this instance is `Body::Struct`.
    pub fn is_struct(&self) -> bool {
        !self.is_enum()
    }
}

impl<V: FromVariant, F> Body<V, F> {
    /// Creates an instance from a slice of `syn::Variant`s.
    pub fn from_variants(variants: &[Variant]) -> Result<Self> {
        let mut v = Vec::with_capacity(variants.len());
        for variant in variants {
            v.push(FromVariant::from_variant(variant)?);
        }

        Ok(Body::Enum(v))
    }
}

impl<V: FromVariant, F: FromField> Body<V, F> {
    /// Creates an instance from an instance of `syn::Body`.
    pub fn from_body(body: &syn::Body) -> Result<Self> {
        match *body {
            syn::Body::Enum(ref variants) => Self::from_variants(variants),
            syn::Body::Struct(ref v_data) => VariantData::from(v_data).map(Body::Struct),
        }
    }
}