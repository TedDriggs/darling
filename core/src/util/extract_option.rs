//! Functions to extract a type `T` from a type declaration `Option<T>`.
//!
//! These functions return `Result` rather than `Option` so they can be used with the `?` operator
//!
//! # Heuristics
//!
//! Since proc-macros don't have access to type information, we have no way of telling if some
//! `Type` is really an `Option`. We can only guess.
//!
//! ```ignore
//! use Vec as Option;
//!
//! #[derive(Serialize)]
//! struct Evil {
//!     // oh no
//!     evil: Vec<String>
//! }
//! ```
//!
//! These functions will return `Ok` if the given [`Type`] is a [`Type::Path`], with a single generic
//! argument, and the last path segment is `"Option"`

use crate::{Error, Result};
use syn::{spanned::Spanned, Type};

/// Extracts `T` from `Option<T>`
///
/// # Errors
///
/// Returns an error if the given type is not an `Option<T>`. For more information, see the [module-level](self) documentation
///
/// # Example
///
/// ```
/// # use darling_core as darling;
/// use darling::util::extract_option;
/// use quote::ToTokens;
/// use syn::Type;
///
/// let ty: Type = syn::parse_str("::std::option::Option<String>")?;
/// let result = extract_option::from_owned(ty)?;
/// let result = result.into_token_stream().to_string();
///
/// assert_eq!(result, "String");
/// # Ok::<(), darling::Error>(())
/// ```
pub fn from_owned(ty: Type) -> Result<Type> {
    let span = ty.span();
    let err = || Error::custom("expected an `Option`").with_span(&span);

    let Type::Path(path) = ty else {
        return Err(err());
    };

    if path.qself.is_some() {
        return Err(err());
    }

    let Some(last_segment) = path.path.segments.last() else {
        return Err(err());
    };

    if last_segment.ident != "Option" {
        return Err(err());
    }

    let syn::PathArguments::AngleBracketed(ty) = last_segment.clone().arguments else {
        return Err(err());
    };

    let args = ty.args.into_iter().collect::<Vec<_>>();

    if args.len() != 1 {
        return Err(err());
    }

    let arg = args
        .into_iter()
        .next()
        .expect("just checked that `.len() == 1`");

    let syn::GenericArgument::Type(ty) = arg else {
        return Err(err());
    };

    Ok(ty)
}

/// Extracts a `&mut Type` `T` from an `Option<T>`
///
/// # Errors
///
/// Returns an error if the given type is not an `Option<T>`. For more information, see the [module-level](self) documentation
///
/// # Example
///
/// ```
/// # use darling_core as darling;
/// use darling::util::extract_option;
/// use quote::ToTokens;
/// use syn::Type;
///
/// let mut ty: Type = syn::parse_str("::std::option::Option<String>")?;
/// let result = extract_option::from_mut(&mut ty)?;
/// let result = result.into_token_stream().to_string();
///
/// assert_eq!(result, "String");
/// # Ok::<(), darling::Error>(())
/// ```
pub fn from_mut(ty: &mut Type) -> Result<&mut Type> {
    let span = ty.span();
    let err = || Error::custom("expected an `Option`").with_span(&span);

    let Type::Path(path) = ty else {
        return Err(err());
    };

    if path.qself.is_some() {
        return Err(err());
    }

    let Some(last_segment) = path.path.segments.last_mut() else {
        return Err(err());
    };

    if last_segment.ident != "Option" {
        return Err(err());
    }

    let syn::PathArguments::AngleBracketed(ty) = &mut last_segment.arguments else {
        return Err(err());
    };

    let args = ty.args.iter_mut().collect::<Vec<_>>();

    if args.len() != 1 {
        return Err(err());
    }

    let arg = args
        .into_iter()
        .next()
        .expect("just checked that `.len() == 1`");

    let syn::GenericArgument::Type(ty) = arg else {
        return Err(err());
    };

    Ok(ty)
}

/// Extracts a `&Type` `T` from an `Option<T>`
///
/// # Errors
///
/// Returns an error if the given type is not an `Option<T>`. For more information, see the [module-level](self) documentation
///
/// # Example
///
/// ```
/// # use darling_core as darling;
/// use darling::util::extract_option;
/// use quote::ToTokens;
/// use syn::Type;
///
/// let ty: Type = syn::parse_str("::std::option::Option<String>")?;
/// let result = extract_option::from_ref(&ty)?;
/// let result = result.into_token_stream().to_string();
///
/// assert_eq!(result, "String");
/// # Ok::<(), darling::Error>(())
/// ```
pub fn from_ref(ty: &Type) -> Result<&Type> {
    let span = ty.span();
    let err = || Error::custom("expected an `Option`").with_span(&span);

    let Type::Path(path) = ty else {
        return Err(err());
    };

    if path.qself.is_some() {
        return Err(err());
    }

    let Some(last_segment) = &path.path.segments.last() else {
        return Err(err());
    };

    if &last_segment.ident != "Option" {
        return Err(err());
    }

    let syn::PathArguments::AngleBracketed(ty) = &last_segment.arguments else {
        return Err(err());
    };

    let args = ty.args.iter().collect::<Vec<_>>();

    if args.len() != 1 {
        return Err(err());
    }

    let arg = args
        .into_iter()
        .next()
        .expect("just checked that `.len() == 1`");

    let syn::GenericArgument::Type(ty) = arg else {
        return Err(err());
    };

    Ok(ty)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::Type;

    macro_rules! test_all {
        ($test:literal, $method:ident) => {
            from_owned(syn::parse_str::<Type>($test).unwrap()).$method();
            from_ref(&syn::parse_str::<Type>($test).unwrap()).$method();
            from_mut(&mut syn::parse_str::<Type>($test).unwrap()).$method();
        };
    }

    // Success

    #[test]
    fn simple() {
        test_all!("Option<String>", unwrap);
    }

    #[test]
    fn fully_qualified() {
        test_all!("std::option::Option<String>", unwrap);
        test_all!("core::option::Option<String>", unwrap);
    }

    #[test]
    fn absolute_path() {
        test_all!("::std::option::Option<String>", unwrap);
        test_all!("::core::option::Option<String>", unwrap);
    }

    #[test]
    fn submodule() {
        test_all!("option::Option<String>", unwrap);
    }

    // Fail

    #[test]
    fn wrong_arg_count() {
        test_all!("Option<String, u8>", unwrap_err);
    }

    #[test]
    fn rejects_qself() {
        test_all!("<T as Option>::Option<u32>", unwrap_err);
    }

    #[test]
    fn non_path() {
        test_all!("&'a Option<u8>", unwrap_err);
    }
}
