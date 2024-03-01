extern crate proc_macro;
use darling::{parse_meta, FromMeta};
use proc_macro::TokenStream;
use syn::ItemFn;

#[derive(Debug, Clone, FromMeta)]
struct MacroArgs {
    #[darling(default)]
    timeout_ms: Option<u16>,
    path: String,
}

pub fn with_explicit_type(args: TokenStream, input: TokenStream) -> TokenStream {
    let _args = parse_meta!(args as MacroArgs);
    let _input = syn::parse_macro_input!(input as ItemFn);

    // do things with `args`
    unimplemented!()
}

pub fn example_with_inferred_type(args: TokenStream, _input: TokenStream) -> TokenStream {
    fn takes_macro_args(args: MacroArgs) -> Option<u16> {
        args.timeout_ms
    }

    let _timeout = takes_macro_args(parse_meta!(args));

    unimplemented!()
}

fn main() {}
