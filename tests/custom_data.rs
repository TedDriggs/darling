use darling::{ast, FromDeriveInput, FromVariant};
use darling_core::FromData;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(from_variants))]
pub struct Container {
    pub data: MyData,
}

#[derive(Default, Debug, FromVariant)]
#[darling(default, attributes(from_variants))]
pub struct Variant {
    into: Option<bool>,
    skip: Option<bool>,
}

#[derive(Debug)]
pub struct MyData {
    pub variants: Vec<Variant>,
}

impl FromData for MyData {
    fn from_data(data: &syn::Data) -> darling_core::Result<Self> {
        let data: ast::Data<Variant, ()> = FromData::from_data(data)?;
        match data {
            ast::Data::Enum(variants) => Ok(MyData { variants }),
            ast::Data::Struct(_) => Err(darling_core::Error::unsupported_shape("struct")),
        }
    }
}

mod source {
    use syn::{parse_quote, DeriveInput};

    pub fn newtype_enum() -> DeriveInput {
        parse_quote! {
            enum Hello {
                World(bool),
                String(String),
            }
        }
    }

    pub fn named_field_enum() -> DeriveInput {
        parse_quote! {
            enum Hello {
                Foo(u16),
                World {
                    name: String
                },
            }
        }
    }

    pub fn empty_enum() -> DeriveInput {
        parse_quote! {
            enum Hello {}
        }
    }

    pub fn named_struct() -> DeriveInput {
        parse_quote! {
            struct Hello {
                world: bool,
            }
        }
    }

    pub fn tuple_struct() -> DeriveInput {
        parse_quote! { struct Hello(String, bool); }
    }
}

#[test]
fn enum_not_struct() {
    // Should pass
    let container = Container::from_derive_input(&source::newtype_enum()).unwrap();
    assert_eq!(container.data.variants.len(), 2);

    let container = Container::from_derive_input(&source::named_field_enum()).unwrap();
    assert_eq!(container.data.variants.len(), 2);

    let container = Container::from_derive_input(&source::empty_enum()).unwrap();
    assert_eq!(container.data.variants.len(), 0);

    // Should error
    Container::from_derive_input(&source::named_struct()).unwrap_err();
    Container::from_derive_input(&source::tuple_struct()).unwrap_err();
}
