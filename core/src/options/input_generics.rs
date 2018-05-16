use syn;

use options::ParseAttribute;
use util::WithOriginal;
use {ast, codegen};
use {Error, FromMeta, Result};

pub type GenericParam = ast::GenericParam<InputTypeParam>;

/// Container for metadata extracted from the `DeriveInput`'s generics.
/// The original generics are preserved as a base for modification during codegen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Generics(WithOriginal<ast::Generics<GenericParam>, syn::Generics>);

impl Generics {
    pub fn as_codegen_generics<'a>(&'a self) -> codegen::Generics<'a> {
        WithOriginal::new(
            ast::Generics {
                params: self.0
                    .parsed
                    .params
                    .iter()
                    .map(as_codegen_generic_param)
                    .collect(),
                where_clause: self.0.parsed.where_clause.as_ref(),
            },
            &self.0.original,
        )
    }

    pub fn from_generics(generics: &syn::Generics) -> Result<Self> {
        Ok(Generics(WithOriginal::new(
            ast::Generics {
                params: generics
                    .params
                    .iter()
                    .map(from_generic_param)
                    .collect::<Result<Vec<_>>>()?,
                where_clause: generics.where_clause.clone(),
            },
            generics.clone(),
        )))
    }
}

fn from_generic_param(param: &syn::GenericParam) -> ::Result<GenericParam> {
    Ok(match *param {
        syn::GenericParam::Type(ref val) => {
            ast::GenericParam::Type(InputTypeParam::from_type_param(val)?)
        }
        syn::GenericParam::Lifetime(ref val) => ast::GenericParam::Lifetime(val.clone()),
        syn::GenericParam::Const(ref val) => ast::GenericParam::Const(val.clone()),
    })
}

fn as_codegen_generic_param<'a>(param: &'a GenericParam) -> codegen::GenericParam<'a> {
    match *param {
        ast::GenericParam::Type(ref val) => ast::GenericParam::Type(val.as_codegen_type_param()),
        ast::GenericParam::Lifetime(ref v) => ast::GenericParam::Lifetime(v),
        ast::GenericParam::Const(ref v) => ast::GenericParam::Const(v),
    }
}

/// Holder for information extracted from type params of the *deriving* type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputTypeParam {
    pub ident: syn::Ident,
    pub bound_attr: Option<ast::TypeParamBounds>,
}

impl InputTypeParam {
    pub fn as_codegen_type_param<'a>(&'a self) -> codegen::TypeParam<'a> {
        codegen::TypeParam {
            ident: &self.ident,
            bound: self.bound_attr.as_ref(),
        }
    }

    fn new(ident: syn::Ident) -> Self {
        InputTypeParam {
            ident,
            bound_attr: None,
        }
    }

    pub fn from_type_param(param: &syn::TypeParam) -> ::Result<Self> {
        let ident = param.ident.clone();
        Self::new(ident).parse_attributes(&param.attrs)
    }
}

impl ParseAttribute for InputTypeParam {
    fn parse_nested(&mut self, mi: &syn::Meta) -> ::Result<()> {
        let name = mi.name().to_string();
        match name.as_str() {
            "bound" => {
                self.bound_attr = FromMeta::from_meta(mi)?;
                Ok(())
            }
            n => Err(Error::unknown_field(n)),
        }
    }
}
