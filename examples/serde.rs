//! Re-implementation of `serde`'s attribute parsing, with a maximally type-safe API
#![allow(unused)]

use darling::{
    ast::{Data, Fields},
    util::{Callable, Flag},
    Error, FromAttributes, FromDeriveInput, FromField as _, FromField, FromMeta, FromVariant,
};
use quote::quote;
use syn::{parse_quote, punctuated::Punctuated, Expr, Generics, Ident, Type, Visibility};

fn main() {
    // Example with struct:

    let input = quote! {
        #[serde(
            rename_all(serialize = "camelCase", deserialize = "snake_case"),
            deny_unknown_fields,
            remote = "ExternalNetworkSettings",
            bound = "T: Validate + Serialize + DeserializeOwned",
            expecting = "a valid network configuration object"
        )]
        struct NetworkConfig<T> {
            #[serde(rename = "auth_token", alias = "api_key", alias = "token")]
            token: String,

            #[serde(default = "default_port", skip_serializing_if = "is_default_port")]
            port: u16,

            #[serde(flatten)]
            extra_options: T,

            #[serde(skip_deserializing, getter = "get_internal_id")]
            internal_id: u64,

            #[serde(with = "custom_duration_parser")]
            timeout: Duration,
        }
    };
    let input = syn::parse2(input).unwrap();
    let input = Input::from_derive_input(&input).unwrap();
    println!("Struct = {input:#?}");

    // Example with enum:

    let input = quote! {
        #[serde(
            tag = "event_type",
            content = "data",
            rename_all = "SCREAMING_SNAKE_CASE",
            crate = "crate::utils::serde_reexport"
        )]
        enum AppEvent {
            #[serde(rename = "USER_LOGIN", alias = "SIGN_IN")]
            Login {
                user_id: Guid,
                #[serde(borrow)]
                session_id: &'a str,
            },

            #[serde(serialize_with = "serialize_upgrade_event")]
            Upgrade {
                old_version: String,
                new_version: String,
            },

            #[serde(untagged)]
            Legacy(LegacyMetadata),

            #[serde(other)]
            UnknownAction,
        }
    };

    let input = syn::parse2(input).unwrap();
    let input = Input::from_derive_input(&input).unwrap();
    // println!("Enum = {input:#?}");
}

/// The final data structure, that contains the entire parsed serde AST
#[derive(Debug)]
enum Input {
    StructUnit(StructUnit),
    StructNamed(StructNamed),
    StructTuple(StructTuple),
    Enum(Enum),
}

impl FromDeriveInput for Input {
    fn from_derive_input(input: &syn::DeriveInput) -> darling::Result<Self> {
        match &input.data {
            syn::Data::Struct(syn::DataStruct {
                fields: syn::Fields::Named(_),
                ..
            }) => StructNamed::from_derive_input(input).map(Self::StructNamed),
            syn::Data::Struct(syn::DataStruct {
                fields: syn::Fields::Unnamed(_),
                ..
            }) => StructTuple::from_derive_input(input).map(Self::StructTuple),
            syn::Data::Struct(syn::DataStruct {
                fields: syn::Fields::Unit,
                ..
            }) => StructUnit::from_derive_input(input).map(Self::StructUnit),
            syn::Data::Enum(data_enum) => Enum::from_derive_input(input).map(Self::Enum),
            syn::Data::Union(_) => Err(Error::custom("unions are not supported")),
        }
    }
}

#[derive(FromDeriveInput, Debug)]
#[darling(attributes(serde))]
struct StructUnit {
    ident: Ident,
    vis: Visibility,
    generics: Generics,
    #[darling(flatten)]
    attr: StructNamedAttr,
}

#[derive(FromDeriveInput, Debug)]
#[darling(attributes(serde))]
struct StructNamed {
    ident: Ident,
    vis: Visibility,
    generics: Generics,
    data: StructNamedFields,
    #[darling(flatten)]
    attr: StructNamedAttr,
}

#[derive(FromDeriveInput, Debug)]
#[darling(attributes(serde))]
struct StructTuple {
    ident: Ident,
    vis: Visibility,
    generics: Generics,
    data: StructTupleFields,
    #[darling(flatten)]
    attr: StructTupleAttr,
}

#[derive(FromDeriveInput, Debug)]
#[darling(attributes(serde))]
struct Enum {
    ident: Ident,
    vis: Visibility,
    generics: Generics,
    data: EnumVariants,
    #[darling(flatten)]
    attr: EnumAttr,
}

#[derive(Debug)]
enum Variant {
    Unit(VariantUnit),
    Named(VariantNamed),
    Tuple(VariantTuple),
}

#[derive(FromField, Debug)]
struct FieldNamed {
    #[darling(with = darling::util::require_ident)]
    ident: Ident,
    vis: Visibility,
    ty: Type,
    #[darling(flatten)]
    attr: FieldAttr,
}

#[derive(FromField, Debug)]
struct FieldTuple {
    vis: Visibility,
    ty: Type,
    #[darling(flatten)]
    attr: FieldAttr,
}

#[derive(FromVariant, Debug)]
struct VariantUnit {
    ident: Ident,
    discriminant: Option<Expr>,
    #[darling(flatten)]
    attr: VariantAttr,
}

#[derive(FromVariant, Debug)]
struct VariantTuple {
    ident: Ident,
    discriminant: Option<Expr>,
    fields: Fields<FieldTuple>,
    #[darling(flatten)]
    attr: VariantAttr,
}

#[derive(FromVariant, Debug)]
struct VariantNamed {
    ident: Ident,
    discriminant: Option<Expr>,
    fields: Fields<FieldNamed>,
    #[darling(flatten)]
    attr: VariantAttr,
}

#[derive(FromMeta, Debug)]
struct StructNamedAttr {
    transparent: Flag,
    tag: Option<WordOr<String>>,
    #[darling(flatten)]
    common: ContainerAttr,
}

#[derive(FromMeta, Debug)]
struct StructTupleAttr {
    transparent: Flag,
    default: Option<DefaultValue>,
    #[darling(flatten)]
    common: ContainerAttr,
}

#[derive(FromMeta, Debug)]
struct EnumAttr {
    tag: Option<String>,
    untagged: Flag,
    content: Option<String>,
    variant_identifier: Flag,
    field_identifier: Flag,
    #[darling(flatten)]
    common: ContainerAttr,
}

/// Attributes applicable to fields
#[derive(FromMeta, Debug)]
struct FieldAttr {
    default: Option<DefaultValue>,
    flatten: Flag,
    skip_serializing_if: Option<Callable>,
    skip_deserializing_if: Option<Callable>,
    getter: Option<Callable>,
    rename: Option<Granular<String>>,
    rename_all: Option<Granular<RenameAll>>,
    skip: Flag,
    skip_serializing: Flag,
    skip_deserializing: Flag,
    serialize_with: Option<Callable>,
    deserialize_with: Option<Callable>,
    with: Option<Callable>,
    #[darling(multiple)]
    alias: Vec<String>,
    borrow: Option<Borrow>,
    bound: Option<Bound>,
}

/// Attributes applicable to variants
#[derive(FromMeta, Debug)]
struct VariantAttr {
    other: Flag,
    untagged: Flag,
    rename: Option<Granular<String>>,
    rename_all: Option<Granular<RenameAll>>,
    skip: Flag,
    skip_serializing: Flag,
    skip_deserializing: Flag,
    serialize_with: Option<Callable>,
    deserialize_with: Option<Callable>,
    with: Option<Callable>,
    #[darling(multiple)]
    alias: Vec<String>,
    borrow: Option<Borrow>,
    bound: Option<Bound>,
}

/// Attributes common for both the struct and enum
#[derive(FromMeta, Debug)]
struct ContainerAttr {
    default: Option<DefaultValue>,
    rename: Option<Granular<String>>,
    rename_all: Option<Granular<RenameAll>>,
    rename_all_fields: Option<Granular<RenameAll>>,
    deny_unknown_fields: Flag,
    bound: Option<Bound>,
    remote: Option<syn::Type>,
    from: Option<syn::Type>,
    try_from: Option<syn::Type>,
    into: Option<syn::Type>,
    #[darling(rename = "crate")]
    krate: Option<syn::Path>,
    expecting: Option<String>,
}

/// #[serde(borrow)] and #[serde(borrow = "'a + 'b + ...")]
#[derive(FromMeta, Debug)]
struct Borrow(WordOr<Punctuated<syn::Lifetime, syn::Token![+]>>);

#[derive(FromMeta, Debug)]
struct Bound(Granular<syn::TypeParam>);

#[derive(FromMeta, Debug)]
struct DefaultValue(WordOr<Callable>);

#[derive(FromMeta, Debug)]
enum RenameAll {
    #[darling(rename = "lowercase")]
    Lowercase,
    #[darling(rename = "UPPERCASE")]
    Uppercase,
    #[darling(rename = "PascalCase")]
    PascalCase,
    #[darling(rename = "camelCase")]
    CamelCase,
    #[darling(rename = "snake_case")]
    SnakeCase,
    #[darling(rename = "SCREAMING_SNAKE_CASE")]
    ScreamingSnakeCase,
    #[darling(rename = "kebab-case")]
    KebabCase,
    #[darling(rename = "SCREAMING-KEBAB-CASE")]
    ScreamingKebabCase,
}

impl TryFrom<&syn::Variant> for Variant {
    type Error = darling::Error;

    fn try_from(variant: &syn::Variant) -> Result<Self, Self::Error> {
        Ok(match variant.fields {
            syn::Fields::Named(_) => Self::Named(VariantNamed::from_variant(variant)?),
            syn::Fields::Unnamed(_) => Self::Tuple(VariantTuple::from_variant(variant)?),
            syn::Fields::Unit => Self::Unit(VariantUnit::from_variant(variant)?),
        })
    }
}

/// For `default: WordOr<String>`, this allows `default` and `default = "four"`
#[derive(Debug, FromMeta)]
#[darling(from_word = || Ok(Self::Word))]
enum WordOr<T> {
    Word,
    Other(T),
}

/// For `rename: Granular<T>`, this allows `rename = "x"` and `rename(serialize = "a", deserialize = "b")`
#[derive(Debug, PartialEq, Eq)]
enum Granular<T> {
    /// Single value decides for both serialization and deserialization
    Both(T),
    /// Fine-grained control over which value is used for serialization or deserialization
    Each {
        serialize: Option<T>,
        deserialize: Option<T>,
    },
}

impl<T: FromMeta> FromMeta for Granular<T> {
    fn from_value(value: &syn::Lit) -> darling::Result<Self> {
        T::from_value(value).map(Self::Both)
    }

    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        #[derive(FromMeta, Debug, PartialEq, Eq)]
        struct GranularEach<T> {
            serialize: Option<T>,
            deserialize: Option<T>,
        }
        GranularEach::from_list(items).map(
            |GranularEach {
                 serialize,
                 deserialize,
             }| Self::Each {
                serialize,
                deserialize,
            },
        )
    }
}

#[derive(Debug)]
struct StructTupleFields {
    fields: Vec<FieldTuple>,
}

impl TryFrom<&syn::Data> for StructTupleFields {
    type Error = darling::Error;

    fn try_from(data: &syn::Data) -> Result<Self, Self::Error> {
        let mut errors = darling::Error::accumulator();
        let syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Unnamed(fields),
            ..
        }) = data
        else {
            unreachable!()
        };
        let fields = fields
            .unnamed
            .iter()
            .filter_map(|field| errors.handle(FieldTuple::from_field(field)))
            .collect();
        errors.finish()?;

        Ok(Self { fields })
    }
}

#[derive(Debug)]
struct StructNamedFields {
    fields: Vec<FieldNamed>,
}

impl TryFrom<&syn::Data> for StructNamedFields {
    type Error = darling::Error;

    fn try_from(data: &syn::Data) -> Result<Self, Self::Error> {
        let mut errors = darling::Error::accumulator();
        let syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(fields),
            ..
        }) = data
        else {
            unreachable!()
        };
        let fields = fields
            .named
            .iter()
            .filter_map(|field| errors.handle(FieldNamed::from_field(field)))
            .collect();
        errors.finish()?;

        Ok(Self { fields })
    }
}

#[derive(Debug)]
struct EnumVariants {
    variants: Vec<Variant>,
}

impl TryFrom<&syn::Data> for EnumVariants {
    type Error = darling::Error;

    fn try_from(data: &syn::Data) -> Result<Self, Self::Error> {
        let mut errors = darling::Error::accumulator();
        let syn::Data::Enum(variants) = data else {
            unreachable!()
        };
        let fields = variants
            .variants
            .iter()
            .filter_map(|field| errors.handle(Variant::try_from(field)))
            .collect();
        errors.finish()?;

        Ok(Self { variants: fields })
    }
}
