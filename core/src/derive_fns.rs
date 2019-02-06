//! Functions to derive `darling`'s traits from well-formed input, without directly depending
//! on `proc_macro`.

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{DeriveInput};

use ::{options, codegen};

/// Run an expression which returns a `darling::Result`, then either return the `Ok` value
/// or return early with compile errors.
macro_rules! check_errors {
    ($e:expr) => {
        match $e {
            Ok(val) => val,
            Err(err) => {
                return err.write_errors().into();
            }
        }
    };
}

/// Create tokens for a `darling::FromMeta` impl from a `DeriveInput`. If
/// the input cannot produce a valid impl, the returned tokens will contain
/// compile errors instead.
pub fn from_meta(input: &DeriveInput) -> TokenStream {
    let container = check_errors!(options::FromMetaOptions::new(input));
    codegen::FromMetaImpl::from(&container).into_token_stream()
}

/// Create tokens for a `darling::FromDeriveInput` impl from a `DeriveInput`. If
/// the input cannot produce a valid impl, the returned tokens will contain
/// compile errors instead.
pub fn from_derive_input(input: &DeriveInput) -> TokenStream {
    let container = check_errors!(options::FdiOptions::new(&input));
    codegen::FromDeriveInputImpl::from(&container).into_token_stream()
}

/// Create tokens for a `darling::FromField` impl from a `DeriveInput`. If
/// the input cannot produce a valid impl, the returned tokens will contain
/// compile errors instead.
pub fn from_field(input: &DeriveInput) -> TokenStream {
    let fdic = check_errors!(options::FromFieldOptions::new(input));
    codegen::FromFieldImpl::from(&fdic).into_token_stream()
}

/// Create tokens for a `darling::FromTypeParam` impl from a `DeriveInput`. If
/// the input cannot produce a valid impl, the returned tokens will contain
/// compile errors instead.
pub fn from_type_param(input: &DeriveInput) -> TokenStream {
    let fdic = check_errors!(options::FromTypeParamOptions::new(input));
    codegen::FromTypeParamImpl::from(&fdic).into_token_stream()
}

/// Create tokens for a `darling::FromVariant` impl from a `DeriveInput`. If
/// the input cannot produce a valid impl, the returned tokens will contain
/// compile errors instead.
pub fn from_variant(input: &DeriveInput) -> TokenStream {
    let fdic = check_errors!(options::FromVariantOptions::new(input));
    codegen::FromVariantImpl::from(&fdic).into_token_stream()
}