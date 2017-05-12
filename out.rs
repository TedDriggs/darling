#![feature(prelude_import)]
#![no_std]
#[prelude_import]
use std::prelude::v1::*;
#[macro_use]
extern crate std as std;
#[macro_use]
extern crate darling;

extern crate syn;

struct Lorem(bool);
impl ::darling::FromMetaItem for Lorem {
    fn from_meta_item(__item: &::syn::MetaItem) -> ::darling::Result<Self> {
        Ok(Lorem(::darling::FromMetaItem::from_meta_item(__item)?))
    }
}

#[test]
pub fn generated() {}
pub mod __test_reexports {
    #[allow(private_in_public)]
    pub use super::generated;
}
pub mod __test {
    extern crate test;
    #[main]
    pub fn main() -> () {
        test::test_main_static(TESTS)
    }
    const TESTS: &'static [self::test::TestDescAndFn] =
        &[self::test::TestDescAndFn {
              desc: self::test::TestDesc {
                  name: self::test::StaticTestName("generated"),
                  ignore: false,
                  should_panic: self::test::ShouldPanic::No,
              },
              testfn: self::test::StaticTestFn(::__test_reexports::generated),
          }];
}
