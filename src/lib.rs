extern crate core;
extern crate attr_deserialize_core;
#[macro_use]
extern crate attr_deserialize_macro;


pub use attr_deserialize_macro::*;

#[doc(inline)]
pub use attr_deserialize_core::FromMetaItem;

pub use attr_deserialize_core::{Result, Error};

#[doc(hidden)]
pub mod export {
    
    pub use ::core::option::Option::{self, Some, None};
    pub use ::core::result::Result::{self, Ok, Err};
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
