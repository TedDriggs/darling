use std::{slice, vec};

use syn;

use usage::{
    self, IdentRefSet, IdentSet, LifetimeRefSet, LifetimeSet, UsesLifetimes, UsesTypeParams,
};
use {Error, FromField, FromVariant, Result};

/// A struct or enum body.
///
/// `V` is the type which receives any encountered variants, and `F` receives struct fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Data<V, F> {
    Enum(Vec<V>),
    Struct(Fields<F>),
}

impl<V, F> Data<V, F> {
    /// Creates an empty body of the same shape as the passed-in body.
    pub fn empty_from(src: &syn::Data) -> Self {
        match *src {
            syn::Data::Enum(_) => Self::Enum(vec![]),
            syn::Data::Struct(ref vd) => Self::Struct(Fields::empty_from(&vd.fields)),
            syn::Data::Union(_) => unreachable!(),
        }
    }

    /// Creates a new `Data<&V, &F>` instance from `Data<V, F>`.
    pub fn as_ref(&self) -> Data<&V, &F> {
        match *self {
            Self::Enum(ref variants) => Data::Enum(variants.iter().collect()),
            Self::Struct(ref data) => Data::Struct(data.as_ref()),
        }
    }

    /// Applies a function `V -> U` on enum variants, if this is an enum.
    pub fn map_enum_variants<T, U>(self, map: T) -> Data<U, F>
    where
        T: FnMut(V) -> U,
    {
        match self {
            Self::Enum(v) => Data::Enum(v.into_iter().map(map).collect()),
            Self::Struct(f) => Data::Struct(f),
        }
    }

    /// Applies a function `F -> U` on struct fields, if this is a struct.
    pub fn map_struct_fields<T, U>(self, map: T) -> Data<V, U>
    where
        T: FnMut(F) -> U,
    {
        match self {
            Self::Enum(v) => Data::Enum(v),
            Self::Struct(f) => Data::Struct(f.map(map)),
        }
    }

    /// Applies a function to the `Fields` if this is a struct.
    pub fn map_struct<T, U>(self, mut map: T) -> Data<V, U>
    where
        T: FnMut(Fields<F>) -> Fields<U>,
    {
        match self {
            Self::Enum(v) => Data::Enum(v),
            Self::Struct(f) => Data::Struct(map(f)),
        }
    }

    /// Consumes the `Data`, returning `Fields<F>` if it was a struct.
    pub fn take_struct(self) -> Option<Fields<F>> {
        match self {
            Self::Enum(_) => None,
            Self::Struct(f) => Some(f),
        }
    }

    /// Consumes the `Data`, returning `Vec<V>` if it was an enum.
    pub fn take_enum(self) -> Option<Vec<V>> {
        match self {
            Self::Enum(v) => Some(v),
            Self::Struct(_) => None,
        }
    }

    /// Returns `true` if this instance is `Data::Enum`.
    pub fn is_enum(&self) -> bool {
        match *self {
            Self::Enum(_) => true,
            Self::Struct(_) => false,
        }
    }

    /// Returns `true` if this instance is `Data::Struct`.
    pub fn is_struct(&self) -> bool {
        !self.is_enum()
    }
}

impl<V: FromVariant, F: FromField> Data<V, F> {
    /// Attempt to convert from a `syn::Data` instance.
    pub fn try_from(body: &syn::Data) -> Result<Self> {
        match *body {
            syn::Data::Enum(ref data) => {
                let mut items = Vec::with_capacity(data.variants.len());
                let mut errors = Vec::new();
                for v_result in data.variants.iter().map(FromVariant::from_variant) {
                    match v_result {
                        Ok(val) => items.push(val),
                        Err(err) => errors.push(err),
                    }
                }

                if !errors.is_empty() {
                    Err(Error::multiple(errors))
                } else {
                    Ok(Self::Enum(items))
                }
            }
            syn::Data::Struct(ref data) => Ok(Self::Struct(Fields::try_from(&data.fields)?)),
            syn::Data::Union(_) => unreachable!(),
        }
    }
}

impl<V: UsesTypeParams, F: UsesTypeParams> UsesTypeParams for Data<V, F> {
    fn uses_type_params<'a>(
        &self,
        options: &usage::Options,
        type_set: &'a IdentSet,
    ) -> IdentRefSet<'a> {
        match *self {
            Self::Struct(ref v) => v.uses_type_params(options, type_set),
            Self::Enum(ref v) => v.uses_type_params(options, type_set),
        }
    }
}

impl<V: UsesLifetimes, F: UsesLifetimes> UsesLifetimes for Data<V, F> {
    fn uses_lifetimes<'a>(
        &self,
        options: &usage::Options,
        lifetimes: &'a LifetimeSet,
    ) -> LifetimeRefSet<'a> {
        match *self {
            Self::Struct(ref v) => v.uses_lifetimes(options, lifetimes),
            Self::Enum(ref v) => v.uses_lifetimes(options, lifetimes),
        }
    }
}

/// Equivalent to `syn::Fields`, but replaces the AST element with a generic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fields<T> {
    pub style: Style,
    pub fields: Vec<T>,
}

impl<T> Fields<T> {
    pub fn empty_from(vd: &syn::Fields) -> Self {
        Self {
            style: vd.into(),
            fields: Vec::new(),
        }
    }

    /// Splits the `Fields` into its style and fields for further processing.
    /// Returns an empty `Vec` for `Unit` data.
    pub fn split(self) -> (Style, Vec<T>) {
        (self.style, self.fields)
    }

    /// Returns true if this variant's data makes it a newtype.
    pub fn is_newtype(&self) -> bool {
        self.style == Style::Tuple && self.len() == 1
    }

    pub fn is_unit(&self) -> bool {
        self.style.is_unit()
    }

    pub fn is_tuple(&self) -> bool {
        self.style.is_tuple()
    }

    pub fn is_struct(&self) -> bool {
        self.style.is_struct()
    }

    pub fn as_ref(&self) -> Fields<&T> {
        Fields {
            style: self.style,
            fields: self.fields.iter().collect(),
        }
    }

    pub fn map<F, U>(self, map: F) -> Fields<U>
    where
        F: FnMut(T) -> U,
    {
        Fields {
            style: self.style,
            fields: self.fields.into_iter().map(map).collect(),
        }
    }

    pub fn iter(&self) -> slice::Iter<T> {
        self.fields.iter()
    }

    /// Returns the number of fields in the structure.
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Returns `true` if the `Fields` contains no fields.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

impl<F: FromField> Fields<F> {
    pub fn try_from(fields: &syn::Fields) -> Result<Self> {
        let (items, errors) = match *fields {
            syn::Fields::Named(ref fields) => {
                let mut items = Vec::with_capacity(fields.named.len());
                let mut errors = Vec::new();

                for field in &fields.named {
                    match FromField::from_field(field) {
                        Ok(val) => items.push(val),
                        Err(err) => errors.push(if let Some(ref ident) = field.ident {
                            err.at(ident)
                        } else {
                            err
                        }),
                    }
                }

                (items, errors)
            }
            syn::Fields::Unnamed(ref fields) => {
                let mut items = Vec::with_capacity(fields.unnamed.len());
                let mut errors = Vec::new();

                for field in &fields.unnamed {
                    match FromField::from_field(field) {
                        Ok(val) => items.push(val),
                        Err(err) => errors.push(if let Some(ref ident) = field.ident {
                            err.at(ident)
                        } else {
                            err
                        }),
                    }
                }

                (items, errors)
            }
            syn::Fields::Unit => (vec![], vec![]),
        };

        if !errors.is_empty() {
            Err(Error::multiple(errors))
        } else {
            Ok(Self {
                style: fields.into(),
                fields: items,
            })
        }
    }
}

impl<T> IntoIterator for Fields<T> {
    type Item = T;
    type IntoIter = vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.fields.into_iter()
    }
}

impl<T> From<Style> for Fields<T> {
    fn from(style: Style) -> Self {
        Self {
            style,
            fields: Vec::new(),
        }
    }
}

impl<T, U: Into<Vec<T>>> From<(Style, U)> for Fields<T> {
    fn from((style, fields): (Style, U)) -> Self {
        style.with_fields(fields)
    }
}

impl<T: UsesTypeParams> UsesTypeParams for Fields<T> {
    fn uses_type_params<'a>(
        &self,
        options: &usage::Options,
        type_set: &'a IdentSet,
    ) -> IdentRefSet<'a> {
        self.fields.uses_type_params(options, type_set)
    }
}

impl<T: UsesLifetimes> UsesLifetimes for Fields<T> {
    fn uses_lifetimes<'a>(
        &self,
        options: &usage::Options,
        lifetimes: &'a LifetimeSet,
    ) -> LifetimeRefSet<'a> {
        self.fields.uses_lifetimes(options, lifetimes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Tuple,
    Struct,
    Unit,
}

impl Style {
    pub fn is_unit(self) -> bool {
        self == Self::Unit
    }

    pub fn is_tuple(self) -> bool {
        self == Self::Tuple
    }

    pub fn is_struct(self) -> bool {
        self == Self::Struct
    }

    /// Creates a new `Fields` of the specified style with the passed-in fields.
    fn with_fields<T, U: Into<Vec<T>>>(self, fields: U) -> Fields<T> {
        Fields {
            style: self,
            fields: fields.into(),
        }
    }
}

impl From<syn::Fields> for Style {
    fn from(vd: syn::Fields) -> Self {
        (&vd).into()
    }
}

impl<'a> From<&'a syn::Fields> for Style {
    fn from(vd: &syn::Fields) -> Self {
        match *vd {
            syn::Fields::Named(_) => Self::Struct,
            syn::Fields::Unnamed(_) => Self::Tuple,
            syn::Fields::Unit => Self::Unit,
        }
    }
}
