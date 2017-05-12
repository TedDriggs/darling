use syn::{Field, Ident, MetaItem};

use {FromMetaItem, Result};
use options::{Body, Core, DefaultExpression, ForwardAttrs, ParseAttribute, ParseBody};
use util::IdentList;

/// Reusable base for `FromDeriveInput`, `FromVariant`, `FromField`, and other top-level 
/// `From*` traits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OuterFrom {
    /// The field on the target struct which should receive the type identifier, if any.
    pub ident: Option<Ident>,

    /// The field on the target struct which should receive the type attributes, if any.
    pub attrs: Option<Ident>,

    pub container: Core,

    /// The attribute names that should be searched.
    pub attr_names: IdentList,

    /// The attribute names that should be forwarded. The presence of the word with no additional 
    /// filtering will cause _all_ attributes to be cloned and exposed to the struct after parsing.
    pub forward_attrs: Option<ForwardAttrs>,

    /// Whether or not the container can be made through conversion from the type `Ident`.
    pub from_ident: bool,
}

impl ParseAttribute for OuterFrom {
    fn parse_nested(&mut self, mi: &MetaItem) -> Result<()> {
        match mi.name() {
            "attributes" => { self.attr_names = FromMetaItem::from_meta_item(mi)?; Ok(()) }
            "forward_attrs" => { self.forward_attrs = FromMetaItem::from_meta_item(mi)?; Ok(()) },
            "from_ident" => {
                // HACK: Declaring that a default is present will cause fields to
                // generate correct code, but control flow isn't that obvious. 
                self.container.default = Some(DefaultExpression::Trait);
                self.from_ident = true; 
                Ok(())
            }
            _ => self.container.parse_nested(mi)
        }
    }
}

impl ParseBody for OuterFrom {
    fn parse_field(&mut self, field: &Field) -> Result<()> {
        self.container.parse_field(field)
    }
}

impl From<(Ident, Body)> for OuterFrom {
    fn from(v: (Ident, Body)) -> Self {
        OuterFrom {
            container: Core::from(v),
            attr_names: Default::default(),
            attrs: Default::default(),
            forward_attrs: Default::default(),
            from_ident: Default::default(),
            ident: Default::default(),
        }
    }
}