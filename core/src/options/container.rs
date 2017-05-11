use ident_case::RenameRule;
use syn;

use {Result, Error, FromMetaItem};
use codegen;
use options::{DefaultExpression, ParseAttribute};

/// A struct or enum which should have `FromMetaItem` or `FromDeriveInput` implementations
/// generated.
#[derive(Debug, Clone)]
pub struct Core {
    /// The type identifier.
    pub ident: syn::Ident,

    /// The type's generics. If the type does not use any generics, this will
    /// be an empty instance.
    pub generics: syn::Generics,

    /// Controls whether missing properties should cause errors or should be filled by
    /// the result of a function call. This can be overridden at the field level.
    pub default: Option<DefaultExpression>,

    /// The rule that should be used to rename all fields/variants in the container.
    pub rename_rule: RenameRule,

    /// An infallible function with the signature `FnOnce(T) -> T` which will be called after the 
    /// target instance is successfully constructed.
    pub map: Option<syn::Path>,
}

impl Core {
    /// Creates a new container, with the identity bound for later diagnostics.
    pub fn new(ident: syn::Ident, generics: syn::Generics, attrs: &[syn::Attribute]) -> Result<Self> {
        (Core {
            ident: ident,
            generics: generics,
            default: None,
            rename_rule: RenameRule::None,
            map: None,
        }).parse_attributes(attrs)
    }

    fn as_codegen_default<'a>(&'a self) -> Option<codegen::DefaultExpression<'a>> {
        self.default.as_ref().map(|expr| {
            match *expr {
                DefaultExpression::Explicit(ref path) => codegen::DefaultExpression::Explicit(path),
                DefaultExpression::Inherit | 
                DefaultExpression::Trait => codegen::DefaultExpression::Trait,
            }
        })
    }
}

impl ParseAttribute for Core {
    fn parse_nested(&mut self, mi: &syn::MetaItem) -> Result<()> {
        match mi.name() {
            "default" => { self.default = FromMetaItem::from_meta_item(mi)?; Ok(()) }
            "rename_all" => { self.rename_rule = FromMetaItem::from_meta_item(mi)?; Ok(()) },
            "map" => { self.map = FromMetaItem::from_meta_item(mi)?; Ok(()) }
            n => Err(Error::unknown_field(n))
        }
    }
}

impl<'a> From<&'a Core> for codegen::TraitImpl<'a> {
    fn from(v: &'a Core) -> Self {
        codegen::TraitImpl {
            ident: &v.ident,
            generics: &v.generics,
            fields: vec![],
            default: v.as_codegen_default(),
            map: v.map.as_ref(),
        }
    }
}

impl<'a> From<&'a Core> for codegen::EnumImpl<'a> {
    fn from(v: &'a Core) -> Self {
        codegen::EnumImpl {
            ident: &v.ident,
            generics: &v.generics,
            variants: Default::default(),
        }
    }
}

impl From<syn::Ident> for Core { 
    fn from(ident: syn::Ident) -> Self {
        Core {
            ident,
            generics: Default::default(),
            default: Default::default(),
            rename_rule: RenameRule::None,
            map: Default::default(),
        }
    }
}