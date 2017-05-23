use quote::{Tokens, ToTokens};
use syn::{Ident, Path, Ty};

use codegen::DefaultExpression;

/// Properties needed to generate code for a field in all the contexts
/// where one may appear.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field<'a> {
    /// The name presented to the user of the library. This will appear
    /// in error messages and will be looked when parsing names.
    pub name_in_attr: &'a str,

    /// The name presented to the author of the library. This will appear
    /// in the setters or temporary variables which contain the values.
    pub ident: &'a Ident,

    /// The type of the field in the input.
    pub ty: &'a Ty,
    pub default_expression: Option<DefaultExpression<'a>>,
    pub with_path: &'a Path,
    pub map: Option<&'a Path>,
    pub skip: bool,
}

impl<'a> Field<'a> {
    pub fn as_declaration(&'a self) -> Declaration<'a> {
        Declaration(self, !self.skip)
    }

    pub fn as_match(&'a self) -> MatchArm<'a> {
        MatchArm(self)
    }

    pub fn as_initializer(&'a self) -> Initializer<'a> {
        Initializer(self)
    }

    pub fn as_applicator(&'a self) -> Applicator<'a> {
        Applicator(self)
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
    fn to_tokens(&self, tokens: &mut Tokens) {
        let ident = self.0.ident;
        let ty = self.0.ty;

        let mutable = if self.1 { quote!(mut) } else { quote!() };

        tokens.append(quote!(
            let #mutable #ident: ::darling::export::Option<#ty> = None;
        ));
    }
}

/// Represents an individual field in the match.
pub struct MatchArm<'a>(&'a Field<'a>);

impl<'a> ToTokens for MatchArm<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        if !self.0.skip {
            let name_str = self.0.name_in_attr;
            let ident = self.0.ident;
            let with_path = self.0.with_path;

            let mut extractor = quote!(#with_path(__inner).map_err(|e| e.at(#name_str))?);
            
            if let Some(ref map) = self.0.map.as_ref() {
                extractor = quote!(#map(#extractor));
            }

            tokens.append(quote!(
                #name_str => {  
                    if #ident.is_none() {
                        #ident = ::darling::export::Some(#extractor);
                    } else {
                        return ::darling::export::Err(::darling::Error::duplicate_field(#name_str));
                    }
                }
            ));
        }
    }
}

/// Adapter which emits tokens to apply a field to `self` if a value
/// was provided.
pub struct Applicator<'a>(&'a Field<'a>);

impl<'a> ToTokens for Applicator<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let ident = self.0.ident;

        tokens.append(quote!(
            if let Some(__override) = #ident {
                self.#ident = __override;
            }
        ))
    }
}

/// Wrapper to generate initialization code for a field.
pub struct Initializer<'a>(&'a Field<'a>);

impl<'a> ToTokens for Initializer<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let name_str = self.0.name_in_attr;
        let ident = self.0.ident;
        if let Some(ref expr) = self.0.default_expression {
            tokens.append(quote!(#ident: match #ident {
                ::darling::export::Some(__val) => __val,
                ::darling::export::None => #expr,
            }));
        } else {
            tokens.append(quote!(#ident: match #ident {
                ::darling::export::Some(__val) => __val,
                ::darling::export::None => 
                    return ::darling::export::Err(::darling::Error::missing_field(#name_str))
            }));
        }
    }
}