//! Types for working with generics during generation phase.

use syn;

use ast;
use util::WithOriginal;

/// Data to generate generics declaration, including per-param bounds overrides
pub type Generics<'a> =
    WithOriginal<ast::Generics<GenericParam<'a>, &'a syn::WhereClause>, &'a syn::Generics>;

pub type GenericParam<'a> =
    ast::GenericParam<TypeParam<'a>, &'a syn::LifetimeDef, &'a syn::ConstParam>;

#[derive(Debug, Clone)]
pub struct TypeParam<'a> {
    pub ident: &'a syn::Ident,
    pub bound: Option<&'a ast::TypeParamBounds>,
}
