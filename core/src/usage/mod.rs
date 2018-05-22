//! Traits and types used for tracking the usage of generic parameters through a proc-macro input.
//!
//! When generating trait impls, libraries often want to automatically figure out which type parameters
//! are used in which fields, and then emit bounds that will produce the most permissive compilable
//! code.

mod type_params;

pub use self::type_params::{CollectTypeParams, UsesTypeParams};
pub use util::{IdentRefSet, IdentSet};
