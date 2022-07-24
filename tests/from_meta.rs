use darling::FromMeta;
use syn::parse_quote;

#[derive(FromMeta)]
struct Meta {
    #[darling(default)]
    meta1: Option<String>,
    #[darling(default)]
    meta2: bool,
}

#[test]
fn nested_meta_meta_value() {
    let meta = Meta::from_list(&vec![parse_quote! {
        meta1 = "thefeature"
    }])
    .unwrap();
    assert_eq!(meta.meta1, Some("thefeature".to_string()));
    assert_eq!(meta.meta2, false);
}

#[test]
fn nested_meta_meta_bool() {
    let meta = Meta::from_list(&vec![parse_quote! {
        meta2
    }])
    .unwrap();
    assert_eq!(meta.meta1, None);
    assert_eq!(meta.meta2, true);
}

#[test]
fn nested_meta_lit_errors() {
    let meta = Meta::from_list(&vec![parse_quote! {
        "meta2"
    }])
    .unwrap();
    assert_eq!(meta.meta1, None);
    assert_eq!(meta.meta2, false);
}
