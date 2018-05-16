use quote::{ToTokens, Tokens};
use syn::{self, GenericParam, Path, TraitBound, TraitBoundModifier, TypeParamBound};

use ast::GenericParamExt;
use codegen::{Generics, TraitImpl};

/// Wrapper for "outer From" traits, such as `FromDeriveInput`, `FromVariant`, and `FromField`.
pub trait OuterFromImpl<'a> {
    /// Gets the path of the trait being implemented.
    fn trait_path(&self) -> Path;

    fn base(&'a self) -> &'a TraitImpl<'a>;

    fn trait_bound(&self) -> Path {
        self.trait_path()
    }

    fn wrap<T: ToTokens>(&'a self, body: T, tokens: &mut Tokens) {
        let base = self.base();
        let trayt = self.trait_path();
        let ty_ident = base.ident;
        let generics =
            remove_generic_attrs(compute_impl_bounds(self.trait_bound(), &base.generics));
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        tokens.append_all(quote!(
            impl #impl_generics #trayt for #ty_ident #ty_generics
                #where_clause
            {
                #body
            }
        ));
    }
}

/// Compute correct generics based on overrides and inherited constraints.
fn compute_impl_bounds(bound: Path, generics: &Generics) -> syn::Generics {
    let Generics {
        ref original,
        ref parsed,
    } = generics;
    let mut working = (*original).clone();

    // There can't be a where clause without params, and an empty param
    // list means we have nothing to do
    if parsed.params.is_empty() {
        return working;
    }

    let added_bound = TypeParamBound::Trait(TraitBound {
        paren_token: None,
        modifier: TraitBoundModifier::None,
        lifetimes: None,
        path: bound,
    });

    for (mut raw, opts) in working.params.iter_mut().zip(parsed.params.iter()) {
        if let &mut GenericParam::Type(ref mut typ) = raw {
            let ty_opts = opts.as_type_param()
                .expect("Original and parsed must be zipped");
            if let Some(bound) = ty_opts.bound {
                typ.bounds = bound.clone().into();
            } else {
                typ.bounds.push(added_bound.clone());
            }
        }
    }

    working
}

/// Emitting attributes in type params on trait impl blocks causes an error,
/// so we strip them until https://github.com/dtolnay/syn/issues/422 is fixed.
fn remove_generic_attrs(mut generics: syn::Generics) -> syn::Generics {
    for param in generics.params.iter_mut() {
        match param {
            syn::GenericParam::Type(ref mut v) => {
                v.attrs.clear();
            }
            syn::GenericParam::Lifetime(ref mut v) => {
                v.attrs.clear();
            }
            syn::GenericParam::Const(ref mut v) => {
                v.attrs.clear();
            }
        }
    }

    generics
}
