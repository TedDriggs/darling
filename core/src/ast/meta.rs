use std::convert::TryFrom;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Ident, Path,
};

use crate::FromMeta;

fn quote_lit<T: Spanned + ToTokens>(val: &T) -> syn::Lit {
    syn::LitStr::new(&self::quote!(#val).to_string(), val.span()).into()
}

/// Convert `self` into a valid `syn::Lit`, preserving the span.
///
/// This trait enables wrapping arbitrary syntax elements in literal strings so that `darling::FromMeta`
/// can operate on them.
pub trait IntoLit {
    /// Convert `self` into a valid literal.
    ///
    /// Implementers should set the span of the returned literal to match that of `self`.
    fn into_lit(self) -> syn::Lit;
}

macro_rules! into_lit {
    ($($ty:path),+ $(,)?) => {
        $(
            impl IntoLit for $ty {
                fn into_lit(self) -> syn::Lit {
                    self.into()
                }
            }
        )*
    };
}

into_lit!(
    syn::Lit,
    syn::LitStr,
    syn::LitByteStr,
    syn::LitByte,
    syn::LitChar,
    syn::LitInt,
    syn::LitFloat,
    syn::LitBool,
);

macro_rules! quote_into_lit {
    ($($ty:path),+ $(,)?) => {
        $(
            impl IntoLit for $ty {
                fn into_lit(self) -> syn::Lit {
                    quote_lit(&self)
                }
            }
        )*
    };
}

quote_into_lit!(Ident, syn::Path, syn::Expr);

/// A [`enum@syn::Meta`] which uses a generic instead of [`enum@syn::Lit`] to accommodate
/// values that are valid Rust, but not valid in `syn::Meta`.
///
/// # Example
/// ```rust,ignore
///
/// // Create a type to parse anything you want to accept to the right of `=` in
/// // a meta item.
/// struct Rhs(Ident);
///
/// impl syn::parse::Parse for Rhs {
///     // impl elided
/// }
///
/// impl IntoLit for Rhs {
///     fn into_lit(self) -> syn::Lit {
///         self.0.into_lit()
///     }
/// }
///
/// #[derive(FromMeta)]
/// struct Receiver {
///     // fields elided
/// }
///
/// fn read_attr(attr: syn::Attribute) -> darling::Result<Receiver> {
///     Meta::<Rhs>::try_from(attr)?.try_parse()
/// }
/// ```
#[derive(Debug, Clone)]
pub enum Meta<T> {
    Path(Path),
    NameValue(MetaNameValue<T>),
    List(MetaList<T>),
}

impl<T: Parse> Meta<T> {
    /// Create a new `Meta` with the given `path` by parsing the `body` for an attribute macro.
    ///
    /// # Example
    /// ```rust,ignore
    /// #[proc_macro_attribute]
    /// pub fn sample(attr: TokenStream, input: TokenStream) -> TokenStream {
    ///     let meta: Meta<YourRhs> = match Meta::with_body(syn::parse_quote!(sample), attr.into()) {
    ///         Ok(m) => m,
    ///         Err(e) => {
    ///             return e.write_errors().into();
    ///         }
    ///     };
    /// }
    /// ```
    ///
    /// The value of `path` does not need to match the name of the macro.
    pub fn parse_body(path: Path, body: TokenStream) -> crate::Result<Self> {
        if body.is_empty() {
            Ok(Self::Path(path))
        } else {
            syn::parse2(self::quote!(#path(#body))).map_err(crate::Error::from)
        }
    }
}

impl<T: Parse + IntoLit> Meta<T> {
    /// Convert `self` into a `syn::Meta`, then call `U::from_meta`.
    pub fn darling_parse<U: FromMeta>(self) -> crate::Result<U> {
        U::from_meta(&self.into())
    }
}

impl From<syn::Meta> for Meta<syn::Lit> {
    fn from(v: syn::Meta) -> Self {
        match v {
            syn::Meta::Path(path) => Self::Path(path),
            syn::Meta::List(list) => Self::List(list.into()),
            syn::Meta::NameValue(nv) => Self::NameValue(nv.into()),
        }
    }
}

impl<T: IntoLit> From<Meta<T>> for syn::Meta {
    fn from(v: Meta<T>) -> Self {
        match v {
            Meta::Path(path) => Self::Path(path),
            Meta::List(list) => Self::List(list.into()),
            Meta::NameValue(nv) => Self::NameValue(nv.into()),
        }
    }
}

impl<T: Parse> Parse for Meta<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Look for paths, allowing for the possibility of keywords as idents
        let path = if input.fork().parse::<Path>().is_ok() {
            input.parse::<Path>()
        } else {
            input.call(Ident::parse_any).map(Path::from)
        }?;

        // Decide which variant is being looked at.
        if input.peek(Token![=]) {
            let eq_token = input.parse::<Token![=]>()?;
            let lit = input.parse::<T>()?;
            Ok(Self::NameValue(MetaNameValue {
                path,
                lit,
                eq_token,
            }))
        } else if input.peek(syn::token::Paren) {
            let content;
            Ok(Self::List(MetaList {
                path,
                paren_token: parenthesized!(content in input),
                nested: content.parse_terminated(NestedMeta::<T>::parse)?,
            }))
        } else {
            Ok(Self::Path(path))
        }
    }
}

/// Try to parse the body of an attribute as `Self`.
impl<T: Parse> TryFrom<syn::Attribute> for Meta<T> {
    type Error = crate::Error;

    fn try_from(value: syn::Attribute) -> Result<Self, Self::Error> {
        let syn::Attribute { path, tokens, .. } = value;
        syn::parse2(quote::quote!(#path #tokens)).map_err(crate::Error::from)
    }
}

/// A [`syn::NestedMeta`] which is generic to accept more than just literals.
#[derive(Debug, Clone)]
pub enum NestedMeta<T> {
    Meta(Meta<T>),
    Lit(T),
}

impl From<syn::NestedMeta> for NestedMeta<syn::Lit> {
    fn from(v: syn::NestedMeta) -> Self {
        match v {
            syn::NestedMeta::Meta(m) => Self::Meta(m.into()),
            syn::NestedMeta::Lit(l) => Self::Lit(l.into()),
        }
    }
}

impl<T: IntoLit> From<NestedMeta<T>> for syn::NestedMeta {
    fn from(v: NestedMeta<T>) -> Self {
        match v {
            NestedMeta::Meta(m) => Self::Meta(m.into()),
            NestedMeta::Lit(l) => Self::Lit(l.into_lit()),
        }
    }
}

/// This will only attempt to parse `Meta<T>`.
impl<T: Parse> Parse for NestedMeta<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse().map(Self::Meta)
    }
}

/// A list of nested meta items.
///
/// This is equivalent to `syn::MetaList`, but is generic over the eventual values that can
/// appear to the right-hand side of the `=` sign.
#[derive(Debug, Clone)]
pub struct MetaList<T> {
    pub path: Path,
    pub paren_token: syn::token::Paren,
    pub nested: Punctuated<NestedMeta<T>, syn::token::Comma>,
}

impl From<syn::MetaList> for MetaList<syn::Lit> {
    fn from(v: syn::MetaList) -> Self {
        Self {
            path: v.path,
            paren_token: v.paren_token,
            nested: v.nested.into_iter().map(NestedMeta::from).collect(),
        }
    }
}

impl<T: IntoLit> From<MetaList<T>> for syn::MetaList {
    fn from(v: MetaList<T>) -> Self {
        syn::MetaList {
            paren_token: v.paren_token,
            path: v.path,
            nested: v.nested.into_iter().map(syn::NestedMeta::from).collect(),
        }
    }
}

/// A [`syn::MetaNameValue`] generic over the type of `lit`.
#[derive(Debug, Clone)]
pub struct MetaNameValue<T> {
    pub path: Path,
    pub eq_token: syn::token::Eq,
    pub lit: T,
}

impl From<syn::MetaNameValue> for MetaNameValue<syn::Lit> {
    fn from(v: syn::MetaNameValue) -> Self {
        Self {
            path: v.path,
            lit: v.lit,
            eq_token: v.eq_token,
        }
    }
}

impl<T: IntoLit> From<MetaNameValue<T>> for syn::MetaNameValue {
    fn from(v: MetaNameValue<T>) -> Self {
        Self {
            eq_token: v.eq_token,
            path: v.path,
            lit: v.lit.into_lit(),
        }
    }
}
