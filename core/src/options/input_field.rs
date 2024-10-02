use std::borrow::Cow;

use quote::format_ident;
use syn::{parse_quote_spanned, spanned::Spanned};

use crate::codegen;
use crate::options::{Core, DefaultExpression, ParseAttribute};
use crate::util::{Flag, SpannedValue};
use crate::{Error, FromMeta, Result};

#[derive(Debug, Clone)]
pub struct InputField {
    pub ident: syn::Ident,
    pub attr_name: Option<String>,
    pub ty: syn::Type,
    pub default: Option<DefaultExpression>,
    pub with: Option<With>,

    /// If `true`, generated code will not look for this field in the input meta item,
    /// instead always falling back to either `InputField::default` or `Default::default`.
    pub skip: Option<SpannedValue<bool>>,
    pub post_transform: Option<codegen::PostfixTransform>,
    pub multiple: Option<bool>,
    pub flatten: Flag,
}

impl InputField {
    /// Generate a view into this field that can be used for code generation.
    pub fn as_codegen_field(&self) -> codegen::Field<'_> {
        codegen::Field {
            ident: &self.ident,
            name_in_attr: self
                .attr_name
                .as_ref()
                .map_or_else(|| Cow::Owned(self.ident.to_string()), Cow::Borrowed),
            ty: &self.ty,
            default_expression: self.as_codegen_default(),
            with_initializer: self.with.as_ref().and_then(With::to_closure_declaration),
            with_path: self.with.as_ref().map(|w| &w.path).map_or_else(
                || {
                    Cow::Owned(
                        parse_quote_spanned!(self.ty.span()=> ::darling::FromMeta::from_meta),
                    )
                },
                Cow::Borrowed,
            ),
            skip: *self.skip.unwrap_or_default(),
            post_transform: self.post_transform.as_ref(),
            multiple: self.multiple.unwrap_or_default(),
            flatten: self.flatten.is_present(),
        }
    }

    /// Generate a codegen::DefaultExpression for this field. This requires the field name
    /// in the `Inherit` case.
    fn as_codegen_default(&self) -> Option<codegen::DefaultExpression<'_>> {
        self.default.as_ref().map(|expr| match *expr {
            DefaultExpression::Explicit(ref path) => codegen::DefaultExpression::Explicit(path),
            DefaultExpression::Inherit => codegen::DefaultExpression::Inherit(&self.ident),
            DefaultExpression::Trait { span } => codegen::DefaultExpression::Trait { span },
        })
    }

    fn new(ident: syn::Ident, ty: syn::Type) -> Self {
        InputField {
            ident,
            ty,
            attr_name: None,
            default: None,
            with: None,
            skip: None,
            post_transform: Default::default(),
            multiple: None,
            flatten: Default::default(),
        }
    }

    pub fn from_field(f: &syn::Field, parent: Option<&Core>) -> Result<Self> {
        let ident = f
            .ident
            .clone()
            .unwrap_or_else(|| syn::Ident::new("__unnamed", ::proc_macro2::Span::call_site()));
        let ty = f.ty.clone();
        let base = Self::new(ident, ty).parse_attributes(&f.attrs)?;

        Ok(if let Some(container) = parent {
            base.with_inherited(container)
        } else {
            base
        })
    }

    /// Apply inherited settings from the container. This is done _after_ parsing
    /// to ensure deference to explicit field-level settings.
    fn with_inherited(mut self, parent: &Core) -> Self {
        // explicit renamings take precedence over rename rules on the container,
        // but in the absence of an explicit name we apply the rule.
        if self.attr_name.is_none() {
            self.attr_name = Some(parent.rename_rule.apply_to_field(self.ident.to_string()));
        }

        // Determine the default expression for this field, based on three pieces of information:
        // 1. Will we look for this field in the attribute?
        // 1. Is there a locally-defined default?
        // 1. Did the parent define a default?
        self.default = match (&self.skip, self.default.is_some(), parent.default.is_some()) {
            // If we have a default, use it.
            (_, true, _) => self.default,

            // If there isn't an explicit default but the struct sets a default, we'll
            // inherit from that.
            (_, false, true) => Some(DefaultExpression::Inherit),

            // If we're skipping the field and no defaults have been expressed then we should
            // use the ::darling::export::Default trait, and set the span to the skip keyword
            // so that an error caused by the skipped field's type not implementing `Default`
            // will correctly identify why darling is trying to use `Default`.
            (Some(v), false, false) if **v => Some(DefaultExpression::Trait { span: v.span() }),

            // If we don't have or need a default, then leave it blank.
            (_, false, false) => None,
        };

        self
    }
}

impl ParseAttribute for InputField {
    fn parse_nested(&mut self, mi: &syn::Meta) -> Result<()> {
        let path = mi.path();

        if path.is_ident("rename") {
            if self.attr_name.is_some() {
                return Err(Error::duplicate_field_path(path).with_span(mi));
            }

            self.attr_name = FromMeta::from_meta(mi)?;

            if self.flatten.is_present() {
                return Err(
                    Error::custom("`flatten` and `rename` cannot be used together").with_span(mi),
                );
            }
        } else if path.is_ident("default") {
            if self.default.is_some() {
                return Err(Error::duplicate_field_path(path).with_span(mi));
            }
            self.default = FromMeta::from_meta(mi)?;
        } else if path.is_ident("with") {
            if self.with.is_some() {
                return Err(Error::duplicate_field_path(path).with_span(mi));
            }

            self.with = Some(With::from_meta(&self.ident, mi)?);

            if self.flatten.is_present() {
                return Err(
                    Error::custom("`flatten` and `with` cannot be used together").with_span(mi),
                );
            }
        } else if path.is_ident("skip") {
            if self.skip.is_some() {
                return Err(Error::duplicate_field_path(path).with_span(mi));
            }

            self.skip = FromMeta::from_meta(mi)?;

            if self.skip.map(|v| *v).unwrap_or_default() && self.flatten.is_present() {
                return Err(
                    Error::custom("`flatten` and `skip` cannot be used together").with_span(mi),
                );
            }
        } else if path.is_ident("map") || path.is_ident("and_then") {
            let transformer = path.get_ident().unwrap().clone();
            if let Some(post_transform) = &self.post_transform {
                if transformer == post_transform.transformer {
                    return Err(Error::duplicate_field_path(path).with_span(mi));
                } else {
                    return Err(Error::custom(format!(
                        "Options `{}` and `{}` are mutually exclusive",
                        transformer, post_transform.transformer
                    ))
                    .with_span(mi));
                }
            }

            self.post_transform = Some(codegen::PostfixTransform::new(
                transformer,
                FromMeta::from_meta(mi)?,
            ));
        } else if path.is_ident("multiple") {
            if self.multiple.is_some() {
                return Err(Error::duplicate_field_path(path).with_span(mi));
            }

            self.multiple = FromMeta::from_meta(mi)?;

            if self.multiple == Some(true) && self.flatten.is_present() {
                return Err(
                    Error::custom("`flatten` and `multiple` cannot be used together").with_span(mi),
                );
            }
        } else if path.is_ident("flatten") {
            if self.flatten.is_present() {
                return Err(Error::duplicate_field_path(path).with_span(mi));
            }

            self.flatten = FromMeta::from_meta(mi)?;

            let mut conflicts = Error::accumulator();

            if self.multiple == Some(true) {
                conflicts.push(
                    Error::custom("`flatten` and `multiple` cannot be used together").with_span(mi),
                );
            }

            if self.attr_name.is_some() {
                conflicts.push(
                    Error::custom("`flatten` and `rename` cannot be used together").with_span(mi),
                );
            }

            if self.with.is_some() {
                conflicts.push(
                    Error::custom("`flatten` and `with` cannot be used together").with_span(mi),
                );
            }

            if self.skip.map(|v| *v).unwrap_or_default() {
                conflicts.push(
                    Error::custom("`flatten` and `skip` cannot be used together").with_span(mi),
                );
            }

            conflicts.finish()?;
        } else {
            return Err(Error::unknown_field_path(path).with_span(mi));
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct With {
    /// The path that generated code should use when calling this.
    path: syn::Path,
    /// If set, the closure that should be assigned to `path` locally.
    closure: Option<syn::ExprClosure>,
}

impl With {
    pub fn from_meta(field_name: &syn::Ident, meta: &syn::Meta) -> Result<Self> {
        if let syn::Meta::NameValue(nv) = meta {
            match &nv.value {
                syn::Expr::Path(path) => Ok(Self::from(path.path.clone())),
                syn::Expr::Closure(closure) => Ok(Self {
                    path: format_ident!("__with_closure_for_{}", field_name).into(),
                    closure: Some(closure.clone()),
                }),
                _ => Err(Error::unexpected_expr_type(&nv.value)),
            }
        } else {
            Err(Error::unsupported_format("non-value"))
        }
    }

    /// Create the statement that declares the closure as a function pointer.
    fn to_closure_declaration(&self) -> Option<syn::Stmt> {
        self.closure.as_ref().map(|c| {
            let path = &self.path;
            // An explicit annotation that the input is borrowed is needed here,
            // or attempting to pass a closure will fail with an issue about a temporary
            // value being dropped while still borrowed in the extractor loop.
            //
            // Making the parameter type explicit here avoids errors if the closure doesn't
            // do enough to make the type clear to the compiler.
            //
            // The explicit return type is needed, or else using `Ok` and `?` in the closure
            // body will produce an error about needing type annotations due to uncertainty
            // about the error variant's type. `T` is left undefined so that postfix transforms
            // still work as expected
            parse_quote_spanned!(c.span()=> let #path: fn(&::syn::Meta) -> ::darling::Result<_> = #c;)
        })
    }
}

impl From<syn::Path> for With {
    fn from(path: syn::Path) -> Self {
        Self {
            path,
            closure: None,
        }
    }
}
