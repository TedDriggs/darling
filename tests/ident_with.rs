use darling::ast;
use darling::FromDeriveInput;
use darling::FromField;
use darling_macro::FromTypeParam;
use darling_macro::FromVariant;
use proc_macro2::Span;
use syn::parse_quote;
use syn::Ident;

#[derive(FromVariant)]
struct Variant {
    #[darling(with = |ident| Ok(format!("!{ident}")))]
    ident: String,
}

#[derive(FromField)]
struct Field {
    #[darling(with = darling::util::require_ident)]
    ident: syn::Ident,
}

#[derive(FromTypeParam)]
struct TypeParam {
    #[darling(with = |ident| Ok(format!("!{ident}")))]
    ident: String,
}

#[derive(FromDeriveInput)]
struct Input {
    #[darling(with = |ident| Ok(format!("!{ident}")))]
    ident: String,
    data: darling::ast::Data<Variant, Field>,
    generics: darling::ast::Generics<ast::GenericParam<TypeParam>>,
}

#[test]
fn field_with() {
    let di = Input::from_derive_input(&parse_quote! {
        struct Demo<T> {
            hello: T
        }
    })
    .unwrap();

    let fields = di.data.take_struct().unwrap();
    let first_field = fields.into_iter().next().unwrap();

    assert_eq!(
        first_field.ident,
        Ident::new("hello", Span::call_site()),
        "FromField"
    );

    assert_eq!(
        di.generics.type_params().next().unwrap().ident,
        "!T",
        "FromTypeParam"
    );

    assert_eq!(di.ident, "!Demo", "FromDeriveInput");

    let di = Input::from_derive_input(&parse_quote! {
        enum Demo {
            Hello
        }
    })
    .unwrap();

    let variants = di.data.take_enum().unwrap();

    assert_eq!(variants.first().unwrap().ident, "!Hello", "FromVariant");
}
