extern crate proc_macro;
use darling::{parse_meta, FromMeta};
use proc_macro::TokenStream;
use syn::ItemFn;

#[derive(Debug, FromMeta)]
struct MacroArgs {
    #[darling(default)]
    timeout_ms: Option<u16>,
    path: String,
}

pub fn your_attr(args: TokenStream, input: TokenStream) -> TokenStream {
    let _args = parse_meta!(args as MacroArgs);
    let _input = syn::parse_macro_input!(input as ItemFn);

    // do things with `args`
    unimplemented!()
}

fn main() {}
