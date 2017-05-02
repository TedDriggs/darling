use syn;

use codegen;
use options::DefaultExpression;

pub struct Container {
    pub ident: syn::Ident,
    pub generics: syn::Generics,
    pub default: Option<DefaultExpression>,
}

impl Container {
    /// Creates a new container, with the identity bound for later diagnostics.
    pub fn new(ident: syn::Ident, generics: syn::Generics) -> Self {
        Container {
            ident: ident,
            generics: generics,
            default: None,
        }
    }

    fn as_codegen_default<'a>(&'a self) -> Option<codegen::DefaultExpression<'a>> {
        self.default.as_ref().map(|expr| {
            match *expr {
                DefaultExpression::Explicit(ref path) => codegen::DefaultExpression::Explicit(path),
                DefaultExpression::InheritFromStruct | 
                DefaultExpression::Trait => codegen::DefaultExpression::Trait,
            }
        })
    }
}

impl<'a> From<&'a Container> for codegen::TraitImpl<'a> {
    fn from(v: &'a Container) -> Self {
        codegen::TraitImpl {
            ident: &v.ident,
            generics: &v.generics,
            fields: vec![],
            default: v.as_codegen_default(),
        }
    }
}