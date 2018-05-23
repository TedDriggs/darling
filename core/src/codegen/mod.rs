use proc_macro2::TokenStream;

mod default_expr;
mod error;
mod field;
mod from_meta_impl;
mod from_derive_impl;
mod from_field;
mod from_type_param;
mod from_variant_impl;
mod outer_from_impl;
mod trait_impl;
mod variant;
mod variant_data;

pub use self::default_expr::DefaultExpression;
pub use self::field::Field;
pub use self::from_meta_impl::FromMetaImpl;
pub use self::from_derive_impl::FromDeriveInputImpl;
pub use self::from_field::FromFieldImpl;
pub use self::from_type_param::FromTypeParamImpl;
pub use self::from_variant_impl::FromVariantImpl;
pub use self::outer_from_impl::OuterFromImpl;
pub use self::trait_impl::TraitImpl;
pub use self::variant::Variant;
pub use self::variant_data::FieldsGen;

use options::ForwardAttrs;
use util::IdentList;

/// Infrastructure for generating an attribute extractor.
pub trait ExtractAttribute {
    fn local_declarations(&self) -> TokenStream;

    fn immutable_declarations(&self) -> TokenStream;

    /// Gets the list of attribute names that should be parsed by the extractor.
    fn attr_names(&self) -> &IdentList;

    fn forwarded_attrs(&self) -> Option<&ForwardAttrs>;

    /// Gets the name used by the generated impl to return to the `syn` item passed as input.
    fn param_name(&self) -> TokenStream;

    /// Gets the core from-meta-item loop that should be used on matching attributes.
    fn core_loop(&self) -> TokenStream;

    fn declarations(&self) -> TokenStream {
        if !self.attr_names().is_empty() {
            self.local_declarations()
        } else {
            self.immutable_declarations()
        }
    }

    /// Generates the main extraction loop.
    fn extractor(&self) -> TokenStream {
        let declarations = self.declarations();

        let will_parse_any = !self.attr_names().is_empty();
        let will_fwd_any = self.forwarded_attrs()
            .map(|fa| !fa.is_empty())
            .unwrap_or_default();

        if !(will_parse_any || will_fwd_any) {
            return quote! {
                #declarations
            };
        }

        let input = self.param_name();

        // The block for parsing attributes whose names have been claimed by the target
        // struct. If no attributes were claimed, this is a pass-through.
        let parse_handled = if will_parse_any {
            let attr_names = self.attr_names().to_strings();
            let core_loop = self.core_loop();
            quote!(
                #(#attr_names)|* => {
                    if let Some(::syn::Meta::List(ref __data)) = __attr.interpret_meta() {
                        let __items = &__data.nested;

                        #core_loop
                    } else {
                        // darling currently only supports list-style
                        continue
                    }
                }
            )
        } else {
            quote!()
        };

        // Specifies the behavior for unhandled attributes. They will either be silently ignored or
        // forwarded to the inner struct for later analysis.
        let forward_unhandled = if will_fwd_any {
            forwards_to_local(self.forwarded_attrs().unwrap())
        } else {
            quote!(_ => continue)
        };

        quote!(
            #declarations
            use ::darling::ToTokens;
            let mut __fwd_attrs: ::darling::export::Vec<::syn::Attribute> = vec![];

            for __attr in &#input.attrs {
                // Filter attributes based on name
                match  ::darling::export::ToString::to_string(&__attr.path.clone().into_token_stream()).as_str() {
                    #parse_handled
                    #forward_unhandled
                }
            }
        )
    }
}

fn forwards_to_local(behavior: &ForwardAttrs) -> TokenStream {
    let push_command = quote!(__fwd_attrs.push(__attr.clone()));
    match *behavior {
        ForwardAttrs::All => quote!(_ => #push_command),
        ForwardAttrs::Only(ref idents) => {
            let names = idents.to_strings();
            quote!(
                #(#names)|* => #push_command,
                _ => continue,
            )
        }
    }
}
