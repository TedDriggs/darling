extern crate core;
extern crate darling_core;

#[allow(unused_imports)]
#[macro_use]
extern crate darling_macro;


pub use darling_macro::*;

#[doc(inline)]
pub use darling_core::FromMetaItem;

pub use darling_core::{Result, Error};

#[doc(hidden)]
pub mod export {
    
    pub use ::core::option::Option::{self, Some, None};
    pub use ::core::result::Result::{self, Ok, Err};
    pub use ::core::default::Default;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
