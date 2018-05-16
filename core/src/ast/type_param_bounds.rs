use quote::{ToTokens, Tokens};
use syn::punctuated::{IntoIter, Iter, Punctuated};
use syn::token::Add;
use syn::{self, TypeParamBound};

use {Error, FromMeta, Result};

/// A set of type param bounds, e.g. `Foo + 'a` in `T: Foo + 'a = ()`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeParamBounds(Punctuated<TypeParamBound, Add>);

impl TypeParamBounds {
    pub fn new(value: Punctuated<TypeParamBound, Add>) -> Self {
        TypeParamBounds(value)
    }

    /// Returns an iterator over borrowed syntax tree nodes of type `&TypeParamBound`.
    pub fn iter<'a>(&'a self) -> Iter<'a, TypeParamBound> {
        self.0.iter()
    }

    /// Returns the number of syntax tree nodes in this punctuated sequence.
    ///
    /// This is the number of `TypeParamBound`s, not the number of `+` signs.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl AsRef<Punctuated<TypeParamBound, Add>> for TypeParamBounds {
    fn as_ref(&self) -> &Punctuated<TypeParamBound, Add> {
        &self.0
    }
}

impl FromMeta for TypeParamBounds {
    fn from_string(value: &str) -> Result<Self> {
        if value == "" {
            return Ok(Default::default());
        }

        // syn can't parse the bound in isolation, so we make a fake type param first
        let type_param: syn::TypeParam = syn::parse_str(&format!("__UNUSED: {}", value))
            .map_err(|_| Error::unknown_value(value))?;

        // We only asked for a bound, so we don't allow a default
        if type_param.default.is_some() {
            return Err(Error::unknown_value(value));
        }

        Ok(TypeParamBounds(type_param.bounds))
    }
}

impl IntoIterator for TypeParamBounds {
    type Item = TypeParamBound;
    type IntoIter = IntoIter<TypeParamBound, Add>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<TypeParamBounds> for Vec<TypeParamBound> {
    fn from(value: TypeParamBounds) -> Self {
        value.0.into_iter().collect()
    }
}

impl From<TypeParamBounds> for Punctuated<TypeParamBound, Add> {
    fn from(value: TypeParamBounds) -> Self {
        value.0
    }
}

impl ToTokens for TypeParamBounds {
    fn to_tokens(&self, tokens: &mut Tokens) {
        self.0.to_tokens(tokens)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error as StdError;

    use super::TypeParamBounds;
    use {Error, FromMeta};

    fn valid(input: &str) -> TypeParamBounds {
        FromMeta::from_string(input).expect("Input can be parsed")
    }

    fn invalid(input: &str) -> Error {
        TypeParamBounds::from_string(input).expect_err("Input should be invalid")
    }

    #[test]
    fn parse_empty() {
        let bound = valid("");
        assert!(bound.is_empty());
    }

    #[test]
    fn parse_single() {
        let bound = valid("Foo");
        assert_eq!(bound.len(), 1);
    }

    #[test]
    fn parse_multiple() {
        let bound = valid("Foo + 'a");
        assert_eq!(bound.len(), 2);
    }

    #[test]
    fn reject_default() {
        let err = invalid("= ()");
        assert_eq!(err.description(), "Unknown literal value");
    }
}
