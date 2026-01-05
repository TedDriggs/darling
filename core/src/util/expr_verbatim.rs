use proc_macro2::TokenStream;
use syn::{parse_quote, Expr};

/// Decodes a darling verbatim expression into the user's input + error that describes
/// why the expression failed to parse
///
/// When the user supplies a `meta` that can't be parsed into a [`syn::Expr`], e.g.:
///
/// #[user_meta(meta = pub(in crate))]
///                    ^^^^^^^^^^^^^
///
/// - We take `pub(in crate)` and feed it as the first parameter to the `Self::from_verbatim` function.
/// - We keep the `syn::Error` from the failed `<syn::Expr as syn::parse::Parse>::parse` and pass that as
///   the second parameter to `from_verbatim`, so we can report a good error in the usual case where user
///   doesn't override `from_verbatim`, which will explain why the user's input is not a valid expression
///
/// In order to supply these 2 `TokenStream`s (input + error) without having to `.clone()` the input
/// (as we would have to do if we used `Expr::Verbatim` to "sneak in" this extra `TokenStream`),
/// we store the input and errors like this:
///
/// ```ignore
/// #[__darling_expr_verbatim(#errors)] __darling_expr_verbatim! { #input }
/// ```
///
/// So in that `pub (in crate)` example, the actual `Expr` that `darling` creates might look
/// something like this:
///
/// ```ignore
/// #[__darling_expr_verbatim(
///      compile_error! { "first error" }
///      compile_error! { "second error" }
/// )]
/// __darling_expr_verbatim! {
///      pub(in crate)
/// }
/// ```
///
/// This function decodes the above expression into the 2 `TokenStream`s, one for the input,
/// and the other for the error:
///
/// ```ignore
/// (
///     TokenStream(
///         pub(in crate)
///     ),
///     TokenStream(
///         compile_error! { "first error" }
///         compile_error! { "second error" }
///     )
/// )
/// ```
///
/// The resulting error `TokenStream` will need to be turned into a [`darling::Error`](Error) with
/// [`Error::from_syn_rendered`] in order to be returned from [`FromMeta::from_verbatim`].
///
/// This function could do that, but it doesn't - because doing so requires cloning the entire error
/// `TokenStream`, which is extra work that doesn't need to be done if you actually need `Verbatim`.
/// For example if you are parsing into [`syn::Visibility`]
pub fn decode(maybe_verbatim: &syn::Expr) -> Option<(&TokenStream, &TokenStream)> {
    let Expr::Macro(ref macr) = maybe_verbatim else {
        return None;
    };
    if !macr.mac.path.is_ident("__darling_expr_verbatim") {
        return None;
    }
    if macr.attrs.len() != 1 {
        return None;
    }
    let meta = &macr
        .attrs
        .first()
        .expect("early return if `len() != 1`")
        .meta;
    if !meta.path().is_ident("__darling_expr_verbatim") {
        return None;
    }
    let err = &meta.require_list().ok()?.tokens;
    let verbatim = &macr.mac.tokens;
    Some((verbatim, err))
}

/// Encodes an arbitrary stream of tokens + error stream into an `syn::Expr`
pub(crate) fn encode(input: TokenStream, error: Option<TokenStream>) -> Expr {
    // For some reason ExprMacro::parse doesn't parse attrs
    let mut mac: syn::ExprMacro = parse_quote!(__darling_expr_verbatim! { #input });
    mac.attrs = vec![parse_quote!(#[__darling_expr_verbatim(#error)])];

    Expr::Macro(mac)
}
