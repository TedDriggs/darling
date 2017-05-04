use std::ops::Deref;

use syn::{Ident, NestedMetaItem, MetaItem};

use {FromMetaItem, Result, Error};

#[derive(Debug, Default)]
pub struct IdentList(Vec<Ident>);

impl Deref for IdentList {
    type Target = Vec<Ident>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromMetaItem for IdentList {
    fn from_list(v: &[NestedMetaItem]) -> Result<Self> {
        let mut idents = Vec::with_capacity(v.len());
        for nmi in v {
            if let NestedMetaItem::MetaItem(MetaItem::Word(ref ident)) = *nmi {
                idents.push(ident.clone());
            } else {
                return Err(Error::unexpected_type("non-word"))
            }
        }

        Ok(IdentList(idents))
    }
}