//! Utility types for attribute parsing.
use std::ops::Deref;

use syn;
use {FromMetaItem, Result};

mod ident_list;
mod ignored;
mod over_ride;

pub use self::ident_list::IdentList;
pub use self::ignored::Ignored;
pub use self::over_ride::Override;

/// Marker type equivalent to `Option<()>` for use in attribute parsing.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Flag(Option<()>);

impl Flag {
    pub fn present() -> Self {
        Flag(Some(()))
    }
}

impl Deref for Flag {
    type Target = Option<()>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromMetaItem for Flag {
    fn from_meta_item(mi: &syn::MetaItem) -> Result<Self> {
        FromMetaItem::from_meta_item(mi).map(Flag)
    }
}

impl PartialEq<Option<()>> for Flag {
    fn eq(&self, rhs: &Option<()>) -> bool {
        self.0 == *rhs
    }
}

impl PartialEq<Flag> for Option<()> {
    fn eq(&self, rhs: &Flag) -> bool {
        *self == rhs.0
    }
}