use syn;

use ::Result;
use codegen;
use options::DefaultExpression;

lazy_static! {
    static ref FROM_META_ITEM: syn::Path = {
        syn::parse_path("::attr_deserialize::FromMetaItem::from_meta_item").unwrap()
    };
}

pub struct Field {
    pub target_name: syn::Ident,
    pub attr_name: Option<syn::Ident>,
    pub ty: syn::Ty,
    pub default: Option<DefaultExpression>,
    pub with: Option<syn::Path>,
}

impl Field {
    pub fn as_codegen_field<'a>(&'a self) -> codegen::Field<'a> {
        codegen::Field {
            name_in_struct: &self.target_name,
            name_in_attr: self.attr_name.as_ref().unwrap_or(&self.target_name).as_ref(),
            ty: &self.ty,
            default_expression: self.as_codegen_default(),
            with_path: self.with.as_ref().unwrap_or(&FROM_META_ITEM),
        }
    }

    fn as_codegen_default<'a>(&'a self) -> Option<codegen::DefaultExpression<'a>> {
        self.default.as_ref().map(|expr| {
            match *expr {
                DefaultExpression::Explicit(ref path) => codegen::DefaultExpression::Explicit(path),
                DefaultExpression::InheritFromStruct => codegen::DefaultExpression::InheritFromStruct(&self.target_name),
                DefaultExpression::Trait => codegen::DefaultExpression::Trait,
            }
        })
    }

    pub fn from_field(f: syn::Field) -> Result<Self> {
        let target_name = f.ident.unwrap();
        let ty = f.ty;
        Ok(Field {
            target_name,
            ty,
            attr_name: None,
            default: None,
            with: None,
        })
    }
}