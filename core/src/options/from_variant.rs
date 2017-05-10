use syn::MetaItem;

use Result;
use options::{OuterFrom, ParseAttribute};

#[derive(Debug)]
pub struct FromVariantOptions {
    pub base: OuterFrom,
}

impl ParseAttribute for FromVariantOptions {
    fn parse_nested(&mut self, mi: &MetaItem) -> Result<()> {
        self.base.parse_nested(mi)
    }
}