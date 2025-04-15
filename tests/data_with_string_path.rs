use darling::FromMeta;
use syn::{parse_quote, Meta};

#[derive(Debug, FromMeta)]
struct Test {
    #[darling(with = "my_parser")]
    value: String,
}

// the custom parser function referred to by string
fn my_parser(meta: &Meta) -> darling::Result<String> {
    match meta {
        Meta::NameValue(nv) => match &nv.value {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) => Ok(s.value()),
            other => Err(darling::Error::unexpected_expr_type(other)),
        },
        _ => Err(darling::Error::unexpected_type("name-value")),
    }
}

#[test]
fn test_data_with_string_path() {
    let meta: Meta = parse_quote!(value = "123");
    let nested = vec![darling::export::NestedMeta::Meta(meta)];
    let parsed = Test::from_list(&nested).unwrap();
    assert_eq!(parsed.value.as_str(), "123");
}
