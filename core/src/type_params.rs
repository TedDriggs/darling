use syn::punctuated::Punctuated;
use syn::{self, Ident, Type};

use util::{IdentRefSet, IdentSet};

/// Searcher for finding type params in a syntax tree.
/// This can be used to determine if a given type parameter needs to be bounded in a generated impl.
pub trait UsesTypeParams {
    /// Returns the subset of the queried type parameters that are used by the implementing syntax element.
    ///
    /// This method only accounts for direct usage by the element; indirect usage via bounds or `where`
    /// predicates are not detected.
    fn uses_type_params<'a>(&self, type_set: &'a IdentSet) -> IdentRefSet<'a>;
}

/// Searcher for finding type params in an iterator.
///
/// This trait extends iterators, providing a way to turn a filtered list of fields or variants into a set
/// of type parameter idents.
pub trait CollectTypeParams {
    /// Consume an iterator, accumulating all type parameters in the elements which occur in `type_set`.
    fn collect_type_params<'a>(self, type_set: &'a IdentSet) -> IdentRefSet<'a>;
}

impl<'i, T, I> CollectTypeParams for T
where
    T: IntoIterator<Item = &'i I>,
    I: 'i + UsesTypeParams,
{
    fn collect_type_params<'a>(self, type_set: &'a IdentSet) -> IdentRefSet<'a> {
        self.into_iter().fold(
            IdentRefSet::with_capacity_and_hasher(type_set.len(), Default::default()),
            |state, value| union_in_place(state, value.uses_type_params(type_set)),
        )
    }
}

/// Collect all type parameters from all items in the provided collection.
fn collect_type_params<'a, 'r, I, T>(iterator: T, type_set: &'a IdentSet) -> IdentRefSet<'a>
where
    T: IntoIterator<Item = &'r I>,
    I: UsesTypeParams + 'r,
{
    iterator.into_iter().fold(
        IdentRefSet::with_capacity_and_hasher(type_set.len(), Default::default()),
        |state, value| union_in_place(state, value.uses_type_params(type_set)),
    )
}

/// Insert the contents of `right` into `left`.
fn union_in_place<'a>(mut left: IdentRefSet<'a>, right: IdentRefSet<'a>) -> IdentRefSet<'a> {
    left.extend(right);

    left
}

impl UsesTypeParams for () {
    fn uses_type_params<'a>(&self, _type_set: &'a IdentSet) -> IdentRefSet<'a> {
        Default::default()
    }
}

impl<T: UsesTypeParams> UsesTypeParams for Option<T> {
    fn uses_type_params<'a>(&self, type_set: &'a IdentSet) -> IdentRefSet<'a> {
        self.as_ref()
            .map(|v| v.uses_type_params(type_set))
            .unwrap_or_default()
    }
}

impl<T: UsesTypeParams> UsesTypeParams for Vec<T> {
    fn uses_type_params<'a>(&self, type_set: &'a IdentSet) -> IdentRefSet<'a> {
        collect_type_params(self.into_iter(), type_set)
    }
}

impl<T: UsesTypeParams, U> UsesTypeParams for Punctuated<T, U> {
    fn uses_type_params<'a>(&self, type_set: &'a IdentSet) -> IdentRefSet<'a> {
        collect_type_params(self.into_iter(), type_set)
    }
}

impl UsesTypeParams for syn::Data {
    fn uses_type_params<'a>(&self, type_set: &'a IdentSet) -> IdentRefSet<'a> {
        match *self {
            syn::Data::Struct(ref v) => v.uses_type_params(type_set),
            syn::Data::Enum(ref v) => v.uses_type_params(type_set),
            // Do unions support generics?
            syn::Data::Union(_) => unimplemented!(),
        }
    }
}

uses_type_params!(syn::Variant, fields);
uses_type_params!(syn::Field, ty);
uses_type_params!(syn::DataStruct, fields);
uses_type_params!(syn::AngleBracketedGenericArguments, args);
uses_type_params!(syn::BareFnArg, ty);
uses_type_params!(syn::QSelf, ty);
uses_type_params!(syn::TypePath, qself, path);
uses_type_params!(syn::TypeBareFn, inputs, output);
uses_type_params!(syn::ParenthesizedGenericArguments, inputs, output);
uses_type_params!(syn::TraitBound, path);
uses_type_params!(syn::TypeTraitObject, bounds);
uses_type_params!(syn::TypeImplTrait, bounds);
uses_type_params!(syn::Binding, ty);

impl UsesTypeParams for syn::DataEnum {
    fn uses_type_params<'a>(&self, type_set: &'a IdentSet) -> IdentRefSet<'a> {
        self.variants.collect_type_params(type_set)
    }
}

impl UsesTypeParams for syn::Fields {
    fn uses_type_params<'a>(&self, type_set: &'a IdentSet) -> IdentRefSet<'a> {
        self.collect_type_params(type_set)
    }
}

/// Check if an Ident exactly matches one of the sought-after type parameters.
impl UsesTypeParams for Ident {
    fn uses_type_params<'a>(&self, type_set: &'a IdentSet) -> IdentRefSet<'a> {
        type_set.iter().filter(|v| *v == self).collect()
    }
}

impl UsesTypeParams for syn::ReturnType {
    fn uses_type_params<'a>(&self, type_set: &'a IdentSet) -> IdentRefSet<'a> {
        if let syn::ReturnType::Type(_, ref ty) = *self {
            ty.uses_type_params(type_set)
        } else {
            Default::default()
        }
    }
}

impl UsesTypeParams for Type {
    fn uses_type_params<'a>(&self, type_set: &'a IdentSet) -> IdentRefSet<'a> {
        match *self {
            Type::Slice(ref v) => v.elem.uses_type_params(type_set),
            Type::Array(ref v) => v.elem.uses_type_params(type_set),
            Type::Ptr(ref v) => v.elem.uses_type_params(type_set),
            Type::Reference(ref v) => v.elem.uses_type_params(type_set),
            Type::BareFn(ref v) => v.uses_type_params(type_set),
            Type::Tuple(ref v) => v.uses_type_params(type_set),
            Type::Path(ref v) => v.uses_type_params(type_set),
            Type::Paren(ref v) => v.elem.uses_type_params(type_set),
            Type::Group(ref v) => v.elem.uses_type_params(type_set),
            Type::TraitObject(ref v) => v.uses_type_params(type_set),
            Type::ImplTrait(ref v) => v.uses_type_params(type_set),
            Type::Macro(_) | Type::Verbatim(_) | Type::Infer(_) | Type::Never(_) => {
                Default::default()
            }
        }
    }
}

uses_type_params!(syn::TypeTuple, elems);

impl UsesTypeParams for syn::Path {
    fn uses_type_params<'a>(&self, type_set: &'a IdentSet) -> IdentRefSet<'a> {
        // Not sure if this is even possible, but a path with no segments definitely
        // can't use type parameters.
        // We don't use `is_empty` here because that also considers punctuation.
        if self.segments.len() == 0 {
            return Default::default();
        }

        // A path segment ident can only match if it is not global and it is the first segment
        // in the path.
        let ident_hits = if !self.global() {
            self.segments[0].ident.uses_type_params(type_set)
        } else {
            Default::default()
        };

        // Merge ident hit, if any, with all hits from path arguments
        self.segments.iter().fold(ident_hits, |state, segment| {
            union_in_place(state, segment.arguments.uses_type_params(type_set))
        })
    }
}

impl UsesTypeParams for syn::PathArguments {
    fn uses_type_params<'a>(&self, type_set: &'a IdentSet) -> IdentRefSet<'a> {
        match *self {
            syn::PathArguments::None => Default::default(),
            syn::PathArguments::AngleBracketed(ref v) => v.uses_type_params(type_set),
            syn::PathArguments::Parenthesized(ref v) => v.uses_type_params(type_set),
        }
    }
}

impl UsesTypeParams for syn::GenericArgument {
    fn uses_type_params<'a>(&self, type_set: &'a IdentSet) -> IdentRefSet<'a> {
        match *self {
            syn::GenericArgument::Type(ref v) => v.uses_type_params(type_set),
            syn::GenericArgument::Binding(ref v) => v.uses_type_params(type_set),
            syn::GenericArgument::Const(_) | syn::GenericArgument::Lifetime(_) => {
                Default::default()
            }
        }
    }
}

impl UsesTypeParams for syn::TypeParamBound {
    fn uses_type_params<'a>(&self, type_set: &'a IdentSet) -> IdentRefSet<'a> {
        match *self {
            syn::TypeParamBound::Trait(ref v) => v.uses_type_params(type_set),
            syn::TypeParamBound::Lifetime(_) => Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::{self, Ident};

    use super::{IdentSet, UsesTypeParams};

    fn given_src(src: &str) -> syn::DeriveInput {
        syn::parse_str(src).unwrap()
    }

    fn ident_set(idents: Vec<&str>) -> IdentSet {
        idents.into_iter().map(Ident::from).collect()
    }

    #[test]
    fn finds_simple() {
        let input = given_src("struct Foo<T, U>(T, i32, A, U);");
        let generics = ident_set(vec!["T", "U", "X"]);
        let matches = input.data.uses_type_params(&generics);
        assert_eq!(matches.len(), 2);
        assert!(matches.contains(&Ident::from("T")));
        assert!(matches.contains(&Ident::from("U")));
        assert!(!matches.contains(&Ident::from("X")));
        assert!(!matches.contains(&Ident::from("A")));
    }

    #[test]
    fn finds_named() {
        let input = given_src(
            r#"
        struct Foo<T, U = usize> {
            bar: T,
            world: U,
        }
        "#,
        );

        let generics = ident_set(vec!["T", "U", "X"]);

        let matches = input.data.uses_type_params(&generics);

        assert_eq!(matches.len(), 2);
        assert!(matches.contains(&Ident::from("T")));
        assert!(matches.contains(&Ident::from("U")));
        assert!(!matches.contains(&Ident::from("X")));
        assert!(!matches.contains(&Ident::from("A")));
    }

    #[test]
    fn finds_as_type_arg() {
        let input = given_src(
            r#"
        struct Foo<T, U> {
            bar: T,
            world: Vec<U>,
        }
        "#,
        );

        let generics = ident_set(vec!["T", "U", "X"]);

        let matches = input.data.uses_type_params(&generics);

        assert_eq!(matches.len(), 2);
        assert!(matches.contains(&Ident::from("T")));
        assert!(matches.contains(&Ident::from("U")));
        assert!(!matches.contains(&Ident::from("X")));
        assert!(!matches.contains(&Ident::from("A")));
    }

    #[test]
    fn associated_type() {
        let input = given_src("struct Foo<'a, T> where T: Iterator { peek: T::Item }");
        let generics = ident_set(vec!["T", "INTO"]);
        let matches = input.data.uses_type_params(&generics);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn box_fn_output() {
        let input = given_src("struct Foo<T>(Box<Fn() -> T>);");
        let generics = ident_set(vec!["T"]);
        let matches = input.data.uses_type_params(&generics);
        assert_eq!(matches.len(), 1);
        assert!(matches.contains(&Ident::from("T")));
    }

    #[test]
    fn box_fn_input() {
        let input = given_src("struct Foo<T>(Box<Fn(&T) -> ()>);");
        let generics = ident_set(vec!["T"]);
        let matches = input.data.uses_type_params(&generics);
        assert_eq!(matches.len(), 1);
        assert!(matches.contains(&Ident::from("T")));
    }

    #[test]
    fn qself_vec() {
        let input = given_src("struct Foo<T>(<Vec<T> as a::b::Trait>::AssociatedItem);");
        let generics = ident_set(vec!["T", "U"]);
        let matches = input.data.uses_type_params(&generics);
        assert_eq!(matches.len(), 1);
        assert!(matches.contains(&Ident::from("T")));
    }
}
