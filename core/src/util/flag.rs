use proc_macro2::Span;
use syn::{spanned::Spanned, Meta};

use crate::{FromMeta, Result};

/// A meta-item that can be present as a word - with no value - or absent.
///
/// Unlike `Option<()>`, `Flag` keeps the span where its word was seen.
/// This enables attaching custom error messages to the word, such as in the
/// case of two conflicting keywords being present.
#[derive(Debug, Clone, Copy, Default)]
pub struct Flag(Option<Span>);

impl Flag {
    /// Creates a new `Flag` which corresponds to the presence of a value.
    pub fn present() -> Self {
        Flag(Some(Span::call_site()))
    }

    /// Check if the flag is present.
    pub fn is_present(&self) -> bool {
        self.0.is_some()
    }

    #[deprecated(since = "0.14.0", note = "Use Flag::is_present")]
    pub fn is_some(&self) -> bool {
        self.is_present()
    }
}

impl FromMeta for Flag {
    fn from_none() -> Option<Self> {
        Some(Flag(None))
    }

    fn from_meta(mi: &syn::Meta) -> Result<Self> {
        if let Meta::Path(p) = mi {
            Ok(Flag(Some(p.span())))
        } else {
            // The implementation for () will produce an error for all non-path meta items;
            // call it to make sure the span behaviors and error messages are the same.
            Err(<()>::from_meta(mi).unwrap_err())
        }
    }
}

impl Spanned for Flag {
    fn span(&self) -> Span {
        self.0.unwrap_or_else(Span::call_site)
    }
}

impl From<Flag> for bool {
    fn from(flag: Flag) -> Self {
        flag.is_present()
    }
}

impl From<bool> for Flag {
    fn from(v: bool) -> Self {
        if v {
            Flag::present()
        } else {
            Flag(None)
        }
    }
}
