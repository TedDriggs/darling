use syn;

use {FromField, Result};

/// A generic container which holds the body of a struct or enum variant.
/// This is an exact match of `syn::VariantData`, but it allows for arbitrary
/// types to support parsing scenarios.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariantData<T> {
    Tuple(Vec<T>),
    Struct(Vec<T>),
    Unit,
}

impl<T> VariantData<T> {
    /// Creates a new `VariantData` instance of the same shape as the one passed in,
    /// but containing no values.
    pub fn empty_from(src: &syn::VariantData) -> Self {
        match *src {
            syn::VariantData::Struct(_) => VariantData::Struct(vec![]),
            syn::VariantData::Tuple(_) => VariantData::Tuple(vec![]),
            syn::VariantData::Unit => VariantData::Unit,
        }
    }

    /// Gets all field declarations. Returns an empty `Vec` for `VariantData::Unit`.
    pub fn fields<'a>(&'a self) -> Vec<&'a T> {
        match *self {
            VariantData::Tuple(ref fields) |
            VariantData::Struct(ref fields) => fields.iter().collect(),
            VariantData::Unit => Vec::new(),
        }
    }

    /// Transforms the fields of the variant by applying `FnMut(T) -> U` to each one.
    pub fn map<F, U>(self, map: F) -> VariantData<U>
        where F: FnMut(T) -> U
    {
        match self {
            VariantData::Tuple(fields) => VariantData::Tuple(fields.into_iter().map(map).collect()),
            VariantData::Struct(fields) => {
                VariantData::Struct(fields.into_iter().map(map).collect())
            }
            VariantData::Unit => VariantData::Unit,
        }
    }

    pub fn as_ref<'a>(&'a self) -> VariantData<&'a T> {
        match *self {
            VariantData::Tuple(ref fields) => VariantData::Tuple(fields.iter().collect()),
            VariantData::Struct(ref fields) => VariantData::Struct(fields.iter().collect()),
            VariantData::Unit => VariantData::Unit,
        }
    }

    /// Returns `true` if this is a unit variant.
    pub fn is_unit(&self) -> bool {
        match *self {
            VariantData::Unit => true,
            _ => false,
        }
    }

    /// Returns `true` if this is a struct-style variant with named fields.
    pub fn is_struct(&self) -> bool {
        match *self {
            VariantData::Struct(_) => true,
            _ => false,
        }
    }

    /// Returns `true` if this is a tuple-style variant with unnamed fields.
    pub fn is_tuple(&self) -> bool {
        match *self {
            VariantData::Tuple(_) => true,
            _ => false,
        }
    }

    /// Returns `true` if this is a newtype-style variant (tuple-style with one field).
    pub fn is_newtype(&self) -> bool {
        match *self {
            VariantData::Tuple(ref fields) => fields.len() == 1,
            _ => false,
        }
    }
}

impl<F: FromField> VariantData<F> {
    pub fn try_from(vd: &syn::VariantData) -> Result<Self> {
        match *vd {
            syn::VariantData::Unit => Ok(VariantData::Unit),
            syn::VariantData::Tuple(ref fields) => {
                let mut f = Vec::with_capacity(fields.len());
                for field in fields {
                    f.push(FromField::from_field(field)?);
                }

                Ok(VariantData::Tuple(f))
            }
            syn::VariantData::Struct(ref fields) => {
                let mut f = Vec::with_capacity(fields.len());
                for field in fields {
                    f.push(FromField::from_field(field)?);
                }

                Ok(VariantData::Struct(f))
            }
        }
    }
}

impl<T> Into<Vec<T>> for VariantData<T> {
    fn into(self) -> Vec<T> {
        match self {
            VariantData::Struct(fields) | VariantData::Tuple(fields) => fields,
            VariantData::Unit => Vec::new(),
        }
    }
}

impl Default for VariantData {
    fn default() -> Self {
        VariantData::Unit
    }
}