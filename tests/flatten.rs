use darling::{util::Flag, FromDeriveInput, FromMeta};
use proc_macro2::Ident;
use syn::parse_quote;

#[derive(FromMeta)]
struct Vis {
    public: Flag,
    private: Flag,
}

#[derive(FromDeriveInput)]
#[darling(attributes(sample))]
struct Example {
    ident: Ident,
    label: String,
    #[darling(flatten)]
    visibility: Vis,
}

#[test]
fn happy_path() {
    let di = Example::from_derive_input(&parse_quote! {
        #[sample(label = "Hello", public)]
        struct Demo {}
    });

    let parsed = di.unwrap();
    assert_eq!(parsed.ident, "Demo");
    assert_eq!(&parsed.label, "Hello");
    assert!(parsed.visibility.public.is_present());
    assert!(!parsed.visibility.private.is_present());
}

#[test]
fn unknown_field_errors() {
    let errors = Example::from_derive_input(&parse_quote! {
        #[sample(label = "Hello", republic)]
        struct Demo {}
    })
    .map(|_| "Should have failed")
    .unwrap_err();

    assert_eq!(errors.len(), 1);
}

/// This test demonstrates flatten being used recursively.
/// Fields are expected to be consumed by the outermost matching struct.
#[test]
fn recursive_flattening() {
    #[derive(FromMeta)]
    struct Nested2 {
        above: isize,
        below: isize,
        port: Option<isize>,
    }

    #[derive(FromMeta)]
    struct Nested1 {
        port: isize,
        starboard: isize,
        #[darling(flatten)]
        z_axis: Nested2,
    }

    #[derive(FromMeta)]
    struct Nested0 {
        fore: isize,
        aft: isize,
        #[darling(flatten)]
        cross_section: Nested1,
    }

    #[derive(FromDeriveInput)]
    #[darling(attributes(boat))]
    struct BoatPosition {
        #[darling(flatten)]
        pos: Nested0,
    }

    let parsed = BoatPosition::from_derive_input(&parse_quote! {
        #[boat(fore = 1, aft = 1, port = 10, starboard = 50, above = 20, below = -3)]
        struct Demo;
    })
    .unwrap();

    assert_eq!(parsed.pos.fore, 1);
    assert_eq!(parsed.pos.aft, 1);

    assert_eq!(parsed.pos.cross_section.port, 10);
    assert_eq!(parsed.pos.cross_section.starboard, 50);

    assert_eq!(parsed.pos.cross_section.z_axis.above, 20);
    assert_eq!(parsed.pos.cross_section.z_axis.below, -3);
    // This should be `None` because the `port` field in `Nested1` consumed
    // the field before the leftovers were passed to `Nested2::from_list`.
    assert_eq!(parsed.pos.cross_section.z_axis.port, None);
}

/// This test confirms that a collection - in this case a HashMap - can
/// be used with `flatten`.
#[test]
fn flattening_into_hashmap() {
    #[derive(FromDeriveInput)]
    #[darling(attributes(ca))]
    struct Catchall {
        hello: String,
        volume: usize,
        #[darling(flatten)]
        others: std::collections::HashMap<String, String>,
    }

    let parsed = Catchall::from_derive_input(&parse_quote! {
        #[ca(hello = "World", volume = 10, first_name = "Alice", second_name = "Bob")]
        struct Demo;
    })
    .unwrap();

    assert_eq!(parsed.hello, "World");
    assert_eq!(parsed.volume, 10);
    assert_eq!(parsed.others.len(), 2);
}
