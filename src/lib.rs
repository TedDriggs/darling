extern crate core;
extern crate darling_core;

#[allow(unused_imports)]
#[macro_use]
extern crate darling_macro;

#[doc(hidden)]
pub use darling_macro::*;

#[doc(inline)]
pub use darling_core::{ApplyMetaItem, FromMetaItem, FromDeriveInput, FromField};

#[doc(inline)]
pub use darling_core::{Result, Error};

#[doc(inline)]
pub use darling_core::util;

/// Core/std trait re-exports. This should help produce generated code which doesn't
/// depend on `std` unnecessarily, and avoids problems caused by aliasing `std` or any
/// of the referenced types.
#[doc(hidden)]
pub mod export {    
    pub use ::core::convert::From;
    pub use ::core::option::Option::{self, Some, None};
    pub use ::core::result::Result::{self, Ok, Err};
    pub use ::core::default::Default;
}
