use proc_macro2::{Delimiter, TokenStream, TokenTree};
use quote::ToTokens;
use syn::{
    ext::IdentExt,
    parse::{discouraged::Speculative, ParseStream, Parser},
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

fn parse_meta_after_path<'a>(path: Path, input: ParseStream<'a>) -> syn::Result<Meta> {
    if input.peek(token::Paren) || input.peek(token::Bracket) || input.peek(token::Brace) {
        parse_meta_list_after_path(path, input).map(Meta::List)
    } else if input.peek(Token![=]) {
        parse_meta_name_value_after_path(path, input).map(Meta::NameValue)
    } else {
        Ok(Meta::Path(path))
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

fn parse_meta_name_value_after_path<'a>(
    path: Path,
    input: ParseStream<'a>,
) -> syn::Result<MetaNameValue> {
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
        input.parse()?
    };
    Ok(MetaNameValue {
        path,
        eq_token,
        value,
    })
}

#[derive(Debug, Clone)]
// Addressing this would break many users of the crate.
#[allow(clippy::large_enum_variant)]
pub enum NestedMeta {
    Meta(syn::Meta),
    Lit(syn::Lit),
}

impl NestedMeta {
    pub fn parse_meta_list(tokens: TokenStream) -> syn::Result<Vec<Self>> {
        syn::punctuated::Punctuated::<NestedMeta, Token![,]>::parse_terminated
            .parse2(tokens)
            .map(|punctuated| punctuated.into_iter().collect())
    }
}

impl syn::parse::Parse for NestedMeta {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        if input.peek(syn::Lit) && !(input.peek(syn::LitBool) && input.peek2(syn::Token![=])) {
            input.parse().map(Self::Lit)
        } else if input.peek(syn::Ident::peek_any) {
            let path = parse_meta_path(input)?;
            parse_meta_after_path(path, input).map(Self::Meta)
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
        }
    }
}
