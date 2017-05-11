use ident_case::RenameRule;
use syn::{MetaItem, Ident, Generics, Attribute};

use Result;
use codegen;
use options::{Core, ParseAttribute, OuterFrom};

#[derive(Debug)]
pub struct FdiOptions {
    pub base: OuterFrom,

    /// The field on the target struct which should receive the type visibility, if any.
    pub vis: Option<Ident>,

    /// The field on the target struct which should receive the type generics, if any.
    pub generics: Option<Ident>,
}

impl FdiOptions {
    pub fn new(ident: Ident, generics: Generics, attrs: &[Attribute]) -> Result<Self> {
        (FdiOptions {
                base: OuterFrom {
                    container: Core {
                        ident: ident,
                        generics: generics,
                        default: None,
                        rename_rule: RenameRule::None,
                        map: Default::default(),
                    },
                    attr_names: Default::default(),
                    attrs: Default::default(),
                    forward_attrs: Default::default(),
                    from_ident: Default::default(),
                    ident: Default::default(),
                },
                vis: Default::default(),
                generics: Default::default(),
            })
            .parse_attributes(attrs)
    }
}

impl<'a> From<&'a FdiOptions> for codegen::FromDeriveInputImpl<'a> {
    fn from(v: &'a FdiOptions) -> Self {
        codegen::FromDeriveInputImpl {
            struct_impl: (&v.base.container).into(),
            attr_names: v.base.attr_names.as_strs(),
            from_ident: Some(v.base.from_ident),
            ident: v.base.ident.as_ref(),
            vis: v.vis.as_ref(),
            generics: v.generics.as_ref(),
            attrs: v.base.attrs.as_ref(),
            forward_attrs: v.base.forward_attrs.as_ref(),
        }
    }
}

impl ParseAttribute for FdiOptions {
    fn parse_nested(&mut self, mi: &MetaItem) -> Result<()> {
        self.base.parse_nested(mi)
    }
}