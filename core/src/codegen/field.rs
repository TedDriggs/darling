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
    pub multiple: bool,
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
        let field: &Field = self.0;
        let ident = field.ident;
        let ty = field.ty;

        let mutable = if self.1 { quote!(mut) } else { quote!() };

        tokens.append(if field.multiple {
            quote!(let #mutable #ident: #ty = ::darling::export::Default::default();)
        } else {
            quote!(let #mutable #ident: ::darling::export::Option<#ty> = None;)
        });
    }
}

/// Represents an individual field in the match.
pub struct MatchArm<'a>(&'a Field<'a>);

impl<'a> ToTokens for MatchArm<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let field: &Field = self.0;
        if !field.skip {
            let name_str = field.name_in_attr;
            let ident = field.ident;
            let with_path = field.with_path;

            /// Errors include the location of the bad input, so we compute that here.
            /// Fields that take multiple values add the index of the error for convenience,
            /// while single-value fields only expose the name in the input attribute.
            let location = if field.multiple {
                quote!(&format!("{}[{}]", #name_str, #ident.len()))
            } else {
                quote!(#name_str)
            };

            let mut extractor = quote!(#with_path(__inner).map_err(|e| e.at(#location))?);
            
            if let Some(ref map) = field.map.as_ref() {
                extractor = quote!(#map(#extractor));
            }

            tokens.append(if field.multiple {
                quote!(
                    #name_str => {
                        #ident.push(#extractor);
                    }
                )
            } else {
                quote!(
                    #name_str => {  
                        if #ident.is_none() {
                            #ident = ::darling::export::Some(#extractor);
                        } else {
                            return ::darling::export::Err(::darling::Error::duplicate_field(#name_str));
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
    fn to_tokens(&self, tokens: &mut Tokens) {
        let field: &Field = self.0;
        let name_str = field.name_in_attr;
        let ident = field.ident;
        tokens.append(if field.multiple {
            if let Some(ref expr) = field.default_expression {
                quote!(#ident: if !#ident.is_empty() {
                    #ident
                } else {
                    #expr
                })
            } else {
                // This could just be `#ident`, but that breaks `cargo expand`, which is 
                // necessary for sanity when working on proc macros.
                //
                // See https://github.com/dtolnay/cargo-expand/issues/14
                quote!(#ident: #ident)
            }
        } else {
            if let Some(ref expr) = field.default_expression {
                quote!(#ident: match #ident {
                    ::darling::export::Some(__val) => __val,
                    ::darling::export::None => #expr,
                })
            } else {
                quote!(#ident: match #ident {
                    ::darling::export::Some(__val) => __val,
                    ::darling::export::None => 
                        return ::darling::export::Err(::darling::Error::missing_field(#name_str))
                })
            }
        });
    }
}