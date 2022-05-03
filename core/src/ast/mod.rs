//! Utility types for working with the AST.
//!
//! Most of these types are generic versions of `syn` types, enabling substitution
//! of arbitrary payloads into specific places in the AST.

mod data;
mod generics;
mod meta;

pub use self::data::*;
pub use self::generics::{GenericParam, GenericParamExt, Generics};
pub use self::meta::{IntoLit, Meta, MetaList, MetaNameValue, NestedMeta};
