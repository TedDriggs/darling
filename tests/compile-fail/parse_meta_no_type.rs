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
    let args = parse_meta!(args);
    let _input = syn::parse_macro_input!(input as ItemFn);

    println!("{:?}", args.timeout_ms);
    unimplemented!()
}

fn main() {}
