use syn::spanned::Spanned;

/// Calls `FromMeta::<#ty>::from_none()` if the `#ty` implements `FromMeta`.
/// if it doesn't, evaluates to an `Option`.
pub fn from_none_call(ty: &syn::Type) -> proc_macro2::TokenStream {
    // If `ty` does not impl FromMeta, the compiler error should point
    // at the offending type rather than at the derive-macro call site.
    quote::quote_spanned!(ty.span() => {
        // Auto-ref specialization, described in detail in the doc
        // comments of the `autoref_specialization` module

        // If `#ty` implements `FromMeta`, `SpecFromMeta::tag()` is used
        #[allow(unused)]
        use _darling::autoref_specialization::SpecFromMeta as _;
        // If it doesn't, `SpecFromMetaAll::tag()` is used
        #[allow(unused)]
        use _darling::autoref_specialization::SpecFromMetaAll as _;
        (&_darling::export::PhantomData::<#ty>).tag().from_none()
    })
}
