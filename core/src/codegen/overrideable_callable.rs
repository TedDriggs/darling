use std::borrow::Cow;

use quote::ToTokens;

use crate::util::Callable;

/// A callable that can be provided by `darling` or by `darling`'s caller.
///
/// Whether or not the callable was produced by `darling` impacts the assumptions
/// code generation can safely make. To preserve spans, an `Option` isn't suitable
/// for this; both paths need a properly-spanned callable, and the consuming code
/// also needs to know if `darling`'s default assumptions about the contents of that
/// item are allowed to hold.
#[derive(Debug, Clone)]
pub enum OverrideableCallable<'a> {
    Custom(Cow<'a, Callable>),
    Default(Cow<'a, Callable>),
}

impl ToTokens for OverrideableCallable<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            OverrideableCallable::Custom(v) | OverrideableCallable::Default(v) => {
                v.to_tokens(tokens)
            }
        }
    }
}
