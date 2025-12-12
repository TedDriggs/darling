use crate::Error;
use syn::Ident;

/// Attaches error message to `Option<Ident>`, turning it into `darling::Result<Ident>`,
/// so it composes nicely with `#[darling(with = ...)]` on a `struct` deriving [`FromField`](crate::FromField)
///
/// Using `#[derive(FromField)]` requires the `ident` field, if present, to be `Option<Ident>`.
/// This is correct if you will use this `struct` as both part of a tuple struct, and a struct with
/// named fields.
///
/// However, if you don't need to do that, then there's no need to wrap it in an `Option`.
///
/// ```ignore
/// #[derive(FromField)]
/// struct Field {
///     // Without #[darling(with)], this must be `ident: Option<syn::Ident>`
///     #[darling(with = darling::util::require_ident)]
///     ident: syn::Ident,
/// }
///
/// #[derive(FromDeriveInput)]
/// struct Input {
///     data: darling::ast::Data<darling_core::util::Ignored, Field>,
/// }
///
/// let input = Input::from_derive_input(&parse_quote! {
///     struct Demo<T> {
///         hello: T
///     }
/// })?;
///
/// let fields = input.data.take_struct()?;
/// let first_field = fields.into_iter().next()?;
///
/// // Not wrapped in `Some`!
/// assert_eq!(first_field.ident, Ident::new("hello", Span::call_site()));
/// ```
pub fn require_ident(ident: Option<Ident>) -> crate::Result<Ident> {
    ident.ok_or_else(|| Error::custom("expected identifier"))
}
