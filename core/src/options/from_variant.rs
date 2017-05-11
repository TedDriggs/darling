use syn::{Ident, MetaItem};

use Result;
use options::{OuterFrom, ParseAttribute};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FromVariantOptions {
    pub base: OuterFrom,
    pub body: Option<Ident>,
}

impl ParseAttribute for FromVariantOptions {
    fn parse_nested(&mut self, mi: &MetaItem) -> Result<()> {
        self.base.parse_nested(mi)
    }
}