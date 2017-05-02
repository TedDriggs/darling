use quote::{Tokens, ToTokens};
use syn::{Ident, Path, Ty};

use codegen::DefaultExpression;

pub struct Field<'a> {
    pub name_in_attr: &'a str,
    pub name_in_struct: &'a Ident,
    pub ty: &'a Ty,
    pub default_expression: Option<DefaultExpression<'a>>,
    pub with_path: &'a Path,
}

impl<'a> Field<'a> {
    pub fn as_var(&'a self) -> FieldVar<'a> {
        FieldVar(self)
    }

    pub fn as_match(&'a self) -> MatchArm<'a> {
        MatchArm(self)
    }

    pub fn as_initializer(&'a self) -> Initializer<'a> {
        Initializer(self)
    }
}

/// An individual field during variable declaration in the generated parsing method.
pub struct FieldVar<'a>(&'a Field<'a>);

impl<'a> ToTokens for FieldVar<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let name_in_struct = self.0.name_in_struct;
        let ty = self.0.ty;

        tokens.append(quote!(
            let mut #name_in_struct: ::darling::export::Option<#ty> = None;
        ));
    }
}

/// Represents an individual field in the match.
pub struct MatchArm<'a>(&'a Field<'a>);

impl<'a> ToTokens for MatchArm<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let name_str = self.0.name_in_attr;
        let name_in_struct = self.0.name_in_struct;
        let with_path = self.0.with_path;

        tokens.append(quote!(
            #name_str => {  
                if #name_in_struct.is_none() {
                    #name_in_struct = ::darling::export::Some(#with_path(__inner)?);
                } else {
                    return ::darling::export::Err(::darling::Error::duplicate_field(#name_str));
                }
            }
        ));
    }
}

/// Wrapper to generate initialization code for a field.
pub struct Initializer<'a>(&'a Field<'a>);

impl<'a> ToTokens for Initializer<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let name_str = self.0.name_in_attr;
        let name_in_struct = self.0.name_in_struct;
        if let Some(ref expr) = self.0.default_expression {
            tokens.append(quote!(#name_in_struct: match #name_in_struct {
                ::darling::export::Some(__val) => __val,
                ::darling::export::None => #expr,
            }));
        } else {
            tokens.append(quote!(#name_in_struct: match #name_in_struct {
                ::darling::export::Some(__val) => __val,
                ::darling::export::None => 
                    return ::darling::export::Err(::darling::Error::missing_field(#name_str))
            }));
        }
    }
}