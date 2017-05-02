use syn;

use codegen;

mod container;
mod field;

pub use self::container::Container;
pub use self::field::Field;

/// A default/fallback expression encountered in attributes during parsing.
pub enum DefaultExpression {
    /// The value should be taken from the `default` instance of the containing struct.
    /// This is not valid in container options.
    InheritFromStruct,
    Explicit(syn::Path),
    Trait,
}