use syn::Attribute;

/// Check whether `Self` would parse the contents of a given attribute in a
/// `FromDeriveInput`, `FromField`, `FromVariant`, `FromAttributes` or other
/// `darling` trait.
pub trait ParsesAttribute {
    /// Returns whether `Self` would parse the given attribute.
    ///
    /// Non-derive attribute proc-macros have full control over output code, and are
    /// required to remove any attributes controlling their functionality to avoid
    /// compiler errors about unknown attributes.
    ///
    /// # Example
    /// ```rust
    /// # use darling_core::{util::WithOriginal, FromAttributes, ParsesAttribute, Result};
    /// # #[allow(dead_code)]
    /// fn consuming_parse<T>(mut input: syn::ItemTrait) -> Result<WithOriginal<T, syn::ItemTrait>>
    /// where
    ///     T: FromAttributes + ParsesAttribute
    /// {
    ///     let parsed = T::from_attributes(&input.attrs);
    ///     input.attrs.retain(|attr| !T::parses(attr));
    ///     Ok(WithOriginal::new(parsed?, input))
    /// }
    /// ```
    fn parses(attribute: &Attribute) -> bool;
}
