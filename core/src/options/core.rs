use ident_case::RenameRule;
use syn;

use {Result, Error, FromMetaItem};
use codegen;
use options::{DefaultExpression, InputField, InputVariant, ParseAttribute, ParseBody};
use util::{Body, VariantData};

/// A struct or enum which should have `FromMetaItem` or `FromDeriveInput` implementations
/// generated.
#[derive(Debug, Clone, PartialEq, Eq)]
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

    /// The body of the _deriving_ type.
    pub body: Body<InputVariant, InputField>,

    /// The custom bound to apply to the generated impl
    pub bound: Option<Vec<syn::WherePredicate>>,
}

impl Core {
    /// Partially initializes `Core` by reading the identity, generics, and body shape.
    pub fn start(di: &syn::DeriveInput) -> Self {
        Core {
            ident: di.ident.clone(),
            generics: di.generics.clone(),
            body: Body::empty_from(&di.body),
            default: Default::default(),
            rename_rule: Default::default(),
            map: Default::default(),
            bound: Default::default(),
        }
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
            "default" => {
                self.default = FromMetaItem::from_meta_item(mi)?;
                Ok(())
            }
            "rename_all" => {
                self.rename_rule = FromMetaItem::from_meta_item(mi)?;
                Ok(())
            }
            "map" => {
                self.map = FromMetaItem::from_meta_item(mi)?;
                Ok(())
            }
            "bound" => {
                self.bound = FromMetaItem::from_meta_item(mi)?;
                Ok(())
            }
            n => Err(Error::unknown_field(n)),
        }
    }
}

impl ParseBody for Core {
    fn parse_variant(&mut self, variant: &syn::Variant) -> Result<()> {
        let v = InputVariant::from_variant(variant, Some(&self))?;

        match self.body {
            Body::Enum(ref mut variants) => {
                variants.push(v);
                Ok(())
            }
            Body::Struct(_) => panic!("Core::parse_variant should never be called for a struct"),
        }
    }

    fn parse_field(&mut self, field: &syn::Field) -> Result<()> {
        let f = InputField::from_field(field, Some(&self))?;

        match self.body {
            Body::Struct(VariantData::Struct(ref mut fields)) |
            Body::Struct(VariantData::Tuple(ref mut fields)) => {
                fields.push(f);
                Ok(())
            }
            Body::Struct(VariantData::Unit) => {
                panic!("Core::parse_field should not be called on unit")
            }
            Body::Enum(_) => panic!("Core::parse_field should never be called for an enum"),
        }
    }
}

impl<'a> From<&'a Core> for codegen::TraitImpl<'a> {
    fn from(v: &'a Core) -> Self {
        codegen::TraitImpl {
            ident: &v.ident,
            generics: &v.generics,
            body: v.body
                .as_ref()
                .map_struct_fields(InputField::as_codegen_field)
                .map_enum_variants(|variant| variant.as_codegen_variant(&v.ident)),
            default: v.as_codegen_default(),
            map: v.map.as_ref(),
            bound: v.bound.as_ref().map(|i| i.as_slice()),
        }
    }
}