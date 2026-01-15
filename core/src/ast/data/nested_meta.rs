use proc_macro2::{Delimiter, TokenStream, TokenTree};
use quote::ToTokens;
use syn::{
    ext::IdentExt,
    parse::{discouraged::Speculative, ParseStream, Parser, StepCursor},
    punctuated::Punctuated,
    token::{self, Brace, Bracket, Paren},
    Expr, ExprLit, Ident, Lit, MacroDelimiter, Meta, MetaList, MetaNameValue, Path, PathSegment,
    Token,
};

fn parse_meta_path<'a>(input: ParseStream<'a>) -> syn::Result<Path> {
    Ok(Path {
        leading_colon: input.parse()?,
        segments: {
            let mut segments = Punctuated::new();
            loop {
                // Allow all identifiers, including keywords.
                if !input.peek(Ident::peek_any) {
                    break;
                }

                let ident = Ident::parse_any(input)?;
                segments.push_value(PathSegment::from(ident));
                if !input.peek(Token![::]) {
                    break;
                }
                let punct = input.parse()?;
                segments.push_punct(punct);
            }
            if segments.is_empty() {
                return Err(input.parse::<Ident>().unwrap_err());
            } else if segments.trailing_punct() {
                return Err(input.error("expected path segment after `::`"));
            }
            segments
        },
    })
}

fn parse_meta_after_path<'a>(path: Path, input: ParseStream<'a>) -> syn::Result<NestedMeta> {
    if input.peek(token::Paren) || input.peek(token::Bracket) || input.peek(token::Brace) {
        parse_meta_list_after_path(path, input)
            .map(Meta::List)
            .map(NestedMeta::Meta)
    } else if input.peek(Token![=]) {
        Ok(match parse_meta_name_value_after_path(path, input)? {
            MetaNameValueAnyRhs::ValidExpr(meta) => NestedMeta::Meta(Meta::NameValue(meta)),
            MetaNameValueAnyRhs::InvalidExpr(meta) => NestedMeta::NameValueInvalidExpr(meta),
        })
    } else {
        Ok(NestedMeta::Meta(Meta::Path(path)))
    }
}

fn parse_meta_list_after_path<'a>(path: Path, input: ParseStream<'a>) -> syn::Result<MetaList> {
    let (delimiter, tokens) = input.step(|cursor| {
        if let Some((TokenTree::Group(g), rest)) = cursor.token_tree() {
            let span = g.delim_span();
            let delimiter = match g.delimiter() {
                Delimiter::Parenthesis => MacroDelimiter::Paren(Paren(span)),
                Delimiter::Brace => MacroDelimiter::Brace(Brace(span)),
                Delimiter::Bracket => MacroDelimiter::Bracket(Bracket(span)),
                Delimiter::None => {
                    return Err(cursor.error("expected delimiter"));
                }
            };
            Ok(((delimiter, g.stream()), rest))
        } else {
            Err(cursor.error("expected delimiter"))
        }
    })?;
    Ok(MetaList {
        path,
        delimiter,
        tokens,
    })
}

enum MetaNameValueAnyRhs {
    /// RHS after `=` is a valid [`syn::Expr`]
    ValidExpr(MetaNameValue),
    /// RHS after `=` is invalid [`syn::Expr`]
    InvalidExpr(MetaNameValueInvalidExpr),
}

fn parse_meta_name_value_after_path<'a>(
    path: Path,
    input: ParseStream<'a>,
) -> syn::Result<MetaNameValueAnyRhs> {
    let eq_token: Token![=] = input.parse()?;
    let ahead = input.fork();
    let lit: Option<Lit> = ahead.parse()?;
    let value = if let (Some(lit), true) = (lit, ahead.is_empty()) {
        input.advance_to(&ahead);
        Expr::Lit(ExprLit {
            attrs: Vec::new(),
            lit,
        })
    } else if input.peek(Token![#]) && input.peek2(token::Bracket) {
        return Err(input.error("unexpected attribute inside of attribute"));
    } else {
        // `input.parse()` advances the original parser, but we
        // want to backtrack in case parsing into an `Expr` fails
        let input_fork = input.fork();

        match input.parse() {
            Ok(expr) => expr,
            // This isn't a valid expression, it might be something like `pub(in crate::module)`
            // so we want to save that and let the user parse it (e.g. into a [`syn::Visibility`]).
            //
            // For more details, see docs of `darling::util::decode_if_verbatim`
            Err(error) => {
                fn eat_until_comma<'c>(
                    cursor: StepCursor<'c, '_>,
                ) -> syn::Result<(TokenStream, syn::buffer::Cursor<'c>)> {
                    let mut rest = *cursor;
                    let mut ts = TokenStream::new();
                    while let Some((tt, next)) = rest.token_tree() {
                        match tt {
                            TokenTree::Punct(punct) if punct.as_char() == ',' => {
                                break;
                            }
                            tt => {
                                ts.extend([tt]);
                                rest = next
                            }
                        }
                    }
                    Ok((ts, rest))
                }

                // Eat everything from the start of the attribute, until the ',' token
                // We'll then parse this into user's custom implementation of `FromMeta::from_verbatim`,
                // if it exists.
                let verbatim_input = input_fork.step(eat_until_comma)?;

                // Advance the original parser past the ',' token, so the next
                // attributes are properly parsed
                input.step(eat_until_comma)?;

                return Ok(MetaNameValueAnyRhs::InvalidExpr(MetaNameValueInvalidExpr {
                    path,
                    eq_token,
                    value: verbatim_input,
                    error: error.into(),
                }));
            }
        }
    };
    Ok(MetaNameValueAnyRhs::ValidExpr(MetaNameValue {
        path,
        eq_token,
        value,
    }))
}

#[derive(Debug, Clone, PartialEq, Eq)]
// Addressing this would break many users of the crate.
#[allow(clippy::large_enum_variant)]
pub enum NestedMeta {
    Meta(syn::Meta),
    Lit(syn::Lit),
    NameValueInvalidExpr(MetaNameValueInvalidExpr),
}

/// Represents a name-value attribute like `foo = ...` where we failed
/// to parse the RHS (`...`) as a [`syn::Expr`].
///
/// For example:
///
/// ```ignore
/// #[processor(vis = pub(crate))]
/// ```
///
/// We first try to parse that  `vis = pub(crate)` into a [`syn::Meta::NameValue`].
/// This requires the RHS (`pub(crate)`) to be a valid [`syn::Expr`].
///
/// Because `pub(crate)` is not a valid expression, we store the [`TokenStream`]
/// corresponding to `pub(crate)` in this struct, as well as the original [`syn::Error`]
/// obtained from <code>\<[syn::Expr] as [Parse](syn::parse::Parse)\>::parse(quote!(pub(crate)))</code>
///
/// This [`syn::Error`] is used for good error messages when a type does not implement
/// [`FromMeta::from_invalid_expr`], as the default implementation of that function just
/// returns the stored error.
#[derive(Debug, Clone)]
pub struct MetaNameValueInvalidExpr {
    /// In `vis = pub(crate)`, this is the `vis`
    pub path: Path,
    /// In `vis = pub(crate)`, this is the `=`
    pub eq_token: Token![=],
    /// The original input that we tried to parse as a `syn::Expr`, but failed
    ///
    /// In `vis = pub(crate)`, this is `pub(crate)`
    pub value: TokenStream,
    /// The error that we got from failing to parse `input` as a `syn::Expr`
    ///
    /// In `vis = pub(crate)`, this is:
    ///
    /// ```ignore
    /// <syn::Expr as syn::parse::Parse>::parse(quote!(pub(crate))).unwrap_err()
    /// ```
    pub error: crate::Error,
}

impl ToTokens for MetaNameValueInvalidExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.path.to_tokens(tokens);
        self.eq_token.to_tokens(tokens);
        self.value.to_tokens(tokens);
    }
}

impl PartialEq for MetaNameValueInvalidExpr {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
            && self.eq_token == other.eq_token
            && self.value.to_string() == other.value.to_string()
    }
}

impl Eq for MetaNameValueInvalidExpr {}

impl NestedMeta {
    pub fn parse_meta_list(tokens: TokenStream) -> syn::Result<Vec<Self>> {
        syn::punctuated::Punctuated::<NestedMeta, Token![,]>::parse_terminated
            .parse2(tokens)
            .map(|punctuated| punctuated.into_iter().collect())
    }
}

impl syn::parse::Parse for NestedMeta {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        // The difference from `syn::Meta` and `NestedMeta`:
        // 1. `syn::Meta` requires a path, named value, or meta list only.
        //   1. `outer(path)`
        //   2. `outer(key = "value")`, the identifier of the key cannot be strict keywords in rust, like `self`, `super`, `crate`, etc.
        //   3. `outer(inner(..))`, the identifier of the inner meta cannot be strict keywords in rust, like `self`, `super`, `crate`, etc.
        // 2. `NestedMeta` allows a literal, which is useful for attributes like `#[outer("foo")]`.
        //   1. `outer("foo")`
        //   2. `outer(42)`
        //   3. `outer(key = "value")`, the identifier of the key can be strict keywords in rust, like `self`, `super`, `crate`, etc.
        //     1. `outer(self = "value")`
        //     2. `outer(type = "Foo")`
        //     3. `outer(crate = "bar")`
        //   4. `outer(inner(..))`, the identifier of the inner meta can be strict keywords in rust, like `self`, `super`, `crate`, etc.
        //     1. `outer(self(..))`
        //     2. `outer(super(..))`
        //     3. `outer(crate(..))`
        if input.peek(syn::Lit) && !(input.peek(syn::LitBool) && input.peek2(syn::Token![=])) {
            input.parse().map(Self::Lit)
        } else if input.peek(syn::Ident::peek_any)
            || input.peek(Token![::]) && input.peek3(syn::Ident::peek_any)
        {
            let path = parse_meta_path(input)?;
            parse_meta_after_path(path, input)
        } else {
            Err(input.error("expected identifier or literal"))
        }
    }
}

impl ToTokens for NestedMeta {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            NestedMeta::Meta(meta) => meta.to_tokens(tokens),
            NestedMeta::Lit(lit) => lit.to_tokens(tokens),
            NestedMeta::NameValueInvalidExpr(meta) => meta.to_tokens(tokens),
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    /// Absolute paths (paths prefixed with `::`) were not recognized as valid
    ///
    /// Issue: <https://github.com/TedDriggs/darling/issues/394>
    #[test]
    fn absolute_path() {
        let input: NestedMeta = parse_quote!(::prost::Message);
        assert_eq!(
            input,
            NestedMeta::Meta(Meta::Path(parse_quote!(::prost::Message)))
        );
    }
}
