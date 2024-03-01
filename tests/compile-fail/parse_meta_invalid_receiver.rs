extern crate proc_macro;
use darling::parse_meta;
use proc_macro::TokenStream;

struct NotMacroArgs;

pub fn invalid_receiver_type(args: TokenStream, _input: TokenStream) -> TokenStream {
    let args = parse_meta!(args as NotMacroArgs);

    unimplemented!()
}

fn main() {}
