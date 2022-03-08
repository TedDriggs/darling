use std::borrow::Cow;

use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt};
use syn::{Ident, Path, PathArguments, Type};

use crate::codegen::{DefaultExpression, PostfixTransform};
use crate::usage::{self, IdentRefSet, IdentSet, UsesTypeParams};

/// Guess if a type is `core::option::Option<T>`.
///
/// There are three requirements:
///
/// 1. The type must be a path
/// 2. The last segment must be `Option`
/// 3. There must be exactly one type argument.
fn is_option(ty: &Type) -> bool {
    if let Type::Path(path) = ty {
        if let Some(last_seg) = path.path.segments.last() {
            if last_seg.ident == "Option" {
                if let PathArguments::AngleBracketed(args) = &last_seg.arguments {
                    return args.args.len() == 1;
                }
            }
        }
    }

    false
}

/// Properties needed to generate code for a field in all the contexts
/// where one may appear.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field<'a> {
    /// The name presented to the user of the library. This will appear
    /// in error messages and will be looked when parsing names.
    pub name_in_attr: Cow<'a, String>,

    /// The name presented to the author of the library. This will appear
    /// in the setters or temporary variables which contain the values.
    pub ident: &'a Ident,

    /// The type of the field in the input.
    pub ty: &'a Type,
    pub default_expression: Option<DefaultExpression<'a>>,
    pub with_path: Cow<'a, Path>,
    pub post_transform: Option<&'a PostfixTransform>,
    pub skip: bool,
    pub multiple: bool,
}

impl<'a> Field<'a> {
    pub fn as_name(&'a self) -> &'a str {
        &self.name_in_attr
    }

    pub fn as_declaration(&'a self) -> Declaration<'a> {
        Declaration(self, !self.skip)
    }

    pub fn as_match(&'a self) -> MatchArm<'a> {
        MatchArm(self)
    }

    pub fn as_initializer(&'a self) -> Initializer<'a> {
        Initializer(self)
    }

    pub fn as_presence_check(&'a self) -> CheckMissing<'a> {
        CheckMissing(self)
    }

    /// Get the behavior of this field if no value is specified, taking into account the field type.
    ///
    /// The struct field `default_expression` is what was explicitly passed in to the `darling` macro.
    /// However, if nothing was passed in _and_ the field's type implies it is meant to be optional,
    /// `darling` will automatically initialize the field to `None` rather than raise an error.
    ///
    /// This transform is done during codegen because passing `Some(DefaultExpression::Trait)` to
    /// the field would be indistinguishable from a field-level `#[darling(default)]`. An explicit
    /// annotation should override container-assigned default values for the field, but one inferred
    /// from type annotations should not.
    fn option_aware_default(&self) -> Option<Cow<'a, DefaultExpression>> {
        if let Some(explicit_default) = self.default_expression.as_ref() {
            Some(Cow::Borrowed(explicit_default))
        } else if is_option(self.ty) {
            Some(Cow::Owned(DefaultExpression::Trait))
        } else {
            None
        }
    }
}

impl<'a> UsesTypeParams for Field<'a> {
    fn uses_type_params<'b>(
        &self,
        options: &usage::Options,
        type_set: &'b IdentSet,
    ) -> IdentRefSet<'b> {
        self.ty.uses_type_params(options, type_set)
    }
}

/// An individual field during variable declaration in the generated parsing method.
pub struct Declaration<'a>(&'a Field<'a>, bool);

impl<'a> Declaration<'a> {
    /// Creates a new declaration with the given field and mutability.
    pub fn new(field: &'a Field<'a>, mutable: bool) -> Self {
        Declaration(field, mutable)
    }
}

impl<'a> ToTokens for Declaration<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let field: &Field = self.0;
        let ident = field.ident;
        let ty = field.ty;

        let mutable = if self.1 { quote!(mut) } else { quote!() };

        tokens.append_all(if field.multiple {
            // This is NOT mutable, as it will be declared mutable only temporarily.
            quote!(let #mutable #ident: #ty = ::darling::export::Default::default();)
        } else {
            quote!(let #mutable #ident: (bool, ::darling::export::Option<#ty>) = (false, None);)
        });
    }
}

/// Represents an individual field in the match.
pub struct MatchArm<'a>(&'a Field<'a>);

impl<'a> ToTokens for MatchArm<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let field: &Field = self.0;
        if !field.skip {
            let name_str = &field.name_in_attr;
            let ident = field.ident;
            let with_path = &field.with_path;
            let post_transform = field.post_transform.as_ref();

            // Errors include the location of the bad input, so we compute that here.
            // Fields that take multiple values add the index of the error for convenience,
            // while single-value fields only expose the name in the input attribute.
            let location = if field.multiple {
                // we use the local variable `len` here because location is accessed via
                // a closure, and the borrow checker gets very unhappy if we try to immutably
                // borrow `#ident` in that closure when it was declared `mut` outside.
                quote!(&format!("{}[{}]", #name_str, __len))
            } else {
                quote!(#name_str)
            };

            // Add the span immediately on extraction failure, so that it's as specific as possible.
            // The behavior of `with_span` makes this safe to do; if the child applied an
            // even-more-specific span, our attempt here will not overwrite that and will only cost
            // us one `if` check.
            let extractor = quote!(#with_path(__inner)#post_transform.map_err(|e| e.with_span(&__inner).at(#location)));

            tokens.append_all(if field.multiple {
                quote!(
                    #name_str => {
                        // Store the index of the name we're assessing in case we need
                        // it for error reporting.
                        let __len = #ident.len();
                        match #extractor {
                            ::darling::export::Ok(__val) => {
                                #ident.push(__val)
                            }
                            ::darling::export::Err(__err) => {
                                __errors.push(__err)
                            }
                        }
                    }
                )
            } else {
                quote!(
                    #name_str => {
                        if !#ident.0 {
                            match #extractor {
                                ::darling::export::Ok(__val) => {
                                    #ident = (true, ::darling::export::Some(__val));
                                }
                                ::darling::export::Err(__err) => {
                                    #ident = (true, None);
                                    __errors.push(__err);
                                }
                            }
                        } else {
                            __errors.push(::darling::Error::duplicate_field(#name_str).with_span(&__inner));
                        }
                    }
                )
            });
        }
    }
}

/// Wrapper to generate initialization code for a field.
pub struct Initializer<'a>(&'a Field<'a>);

impl<'a> ToTokens for Initializer<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let field: &Field = self.0;
        let ident = field.ident;
        tokens.append_all(if field.multiple {
            if let Some(ref expr) = field.default_expression {
                quote!(#ident: if !#ident.is_empty() {
                    #ident
                } else {
                    #expr
                })
            } else {
                quote!(#ident: #ident)
            }
        } else if let Some(ref expr) = self.0.option_aware_default() {
            quote!(#ident: match #ident.1 {
                ::darling::export::Some(__val) => __val,
                ::darling::export::None => #expr,
            })
        } else {
            quote!(#ident: #ident.1.expect("Uninitialized fields without defaults were already checked"))
        });
    }
}

/// Creates an error if a field has no value and no default.
pub struct CheckMissing<'a>(&'a Field<'a>);

impl<'a> ToTokens for CheckMissing<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let field = self.0;

        if !field.multiple && field.option_aware_default().is_none() {
            let ident = field.ident;
            let name_in_attr = &field.name_in_attr;

            tokens.append_all(quote! {
                if !#ident.0 {
                    __errors.push(::darling::Error::missing_field(#name_in_attr));
                }
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::is_option;
    use syn::parse_quote;

    #[test]
    fn is_option_simple() {
        assert!(is_option(&parse_quote!(Option<Foo>)));
    }

    #[test]
    fn is_option_path() {
        assert!(is_option(&parse_quote!(::std::option::Option<(i32, i64)>)));
        assert!(is_option(&parse_quote!(::core::option::Option<[i32; 4]>)));
    }

    #[test]
    fn is_option_result() {
        assert!(!is_option(&parse_quote!(Result<Example, Error>)));
    }
}
