use ident_case::RenameRule;
use syn::{MetaItem, Ident, Generics, Attribute};

use {Result, FromMetaItem};
use codegen;
use options::{Container, ForwardAttrs, ParseAttribute, DefaultExpression};
use util::IdentList;

#[derive(Debug)]
pub struct FromDeriveInputContainer {
    pub container: Container,

    /// The attribute names that should be searched.
    pub attr_names: IdentList,

    /// The field on the target struct which should receive the type identifier, if any.
    pub ident: Option<Ident>,

    /// The field on the target struct which should receive the type visibility, if any.
    pub vis: Option<Ident>,

    /// The field on the target struct which should receive the type generics, if any.
    pub generics: Option<Ident>,

    /// The attribute names that should be forwarded. The presence of the word with no additional 
    /// filtering will cause _all_ attributes to be cloned and exposed to the struct after parsing.
    pub forward_attrs: Option<ForwardAttrs>,

    /// The field on the target struct which should receive the type attributes, if any.
    pub attrs: Option<Ident>,

    /// Whether or not the container can be made through conversion from the type `Ident`.
    pub from_ident: bool,
}

impl FromDeriveInputContainer {
    pub fn new(ident: Ident, generics: Generics, attrs: &[Attribute]) -> Result<Self> {
        (FromDeriveInputContainer {
            container: Container {
                ident: ident,
                generics: generics,
                default: None,
                rename_rule: RenameRule::None,
                map: Default::default(),
            },
            attr_names: Default::default(),
            ident: Default::default(),
            vis: Default::default(),
            generics: Default::default(),
            attrs: Default::default(),
            forward_attrs: Default::default(),
            from_ident: Default::default(),
        }).parse_attributes(attrs)
    }
}

impl<'a> From<&'a FromDeriveInputContainer> for codegen::FromDeriveInputImpl<'a> {
    fn from(v: &'a FromDeriveInputContainer) -> Self {
        codegen::FromDeriveInputImpl {
            struct_impl: (&v.container).into(),
            attr_names: v.attr_names.as_strs(),
            from_ident: Some(v.from_ident),
            ident: v.ident.as_ref(),
            vis: v.vis.as_ref(),
            generics: v.generics.as_ref(),
            attrs: v.attrs.as_ref(),
            forward_attrs: v.forward_attrs.as_ref(),
        }
    }
}

impl ParseAttribute for FromDeriveInputContainer {
    fn parse_nested(&mut self, mi: &MetaItem) -> Result<()> {
        match mi.name() {
            "attributes" => { self.attr_names = FromMetaItem::from_meta_item(mi)?; Ok(()) },
            "forward_attrs" => { self.forward_attrs = FromMetaItem::from_meta_item(mi)?; Ok(()) },
            "from_ident" => {
                // HACK: Declaring that a default is present will cause fields to
                // generate correct code, but control flow isn't that obvious. 
                self.container.default = Some(DefaultExpression::Trait);
                self.from_ident = true; 
                Ok(()) 
            }
            _ => { self.container.parse_nested(mi) }
        }
    }
}