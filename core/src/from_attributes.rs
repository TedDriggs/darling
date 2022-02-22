use syn::Attribute;

use crate::Result;

/// Create an instance by parsing a list of attributes.
///
/// This trait is useful when dealing with items such as traits on traits and impl blocks,
/// for which `darling` does not provide dedicated traits.
///
/// # Read Before Using
/// Prefer traits such as `FromDeriveInput` or `FromField` over this trait when possible.
/// As shown below, this trait requires additional code to usefully plug into a proc-macro,
/// as it will require traversing the `syn` input explicitly.
///
/// # Example
/// This is an _illustrative_ example of using `FromAttributes` for a real use-case, adding
/// tracing and authorization to methods
///
/// ```rust,ignore
/// #[derive(Default, FromAttributes)]
/// #[darling(attributes(aspect), default)]
/// pub struct MethodOpts {
///     trace: bool,
///     role: Option<string>,
/// }
///
/// pub struct AspectOpts {
///     methods: util::WithOriginal<MethodOpts, ImplItemMethod>
/// }
///
/// impl TryFrom<ItemImpl> for AspectOps {
///     type Error = darling::Error;
///     
///     fn try_from(item_impl: ItemImpl) -> darling::Result<Self> {
///         let mut methods = vec![];
///         let mut errors = vec![];
///         for item in item_impl.items {
///             if let ImplItem::Method(method) = item {
///                 match MethodOpts::from_attributes(&method.attrs) {
///                     Ok(opts) => methods.push(WithOriginal::new(opts, method)),
///                     Err(e) => errors.push(e),
///                 }
///             }
///         }
///
///         if !errors.is_empty() {
///             return Err(darling::Error::multiple(errors));
///         }
///
///         Ok(Self { methods })
///     }
/// }
///
/// #[proc_macro_attribute]
/// pub fn aspect(input: TokenStream) -> TokenStream {
///     let impl_item: ItemImpl = syn::parse_macro_input!(input);
///     let options = match AspectOps::try_from(impl_item) {
///         Ok(ops) => ops,
///         Err(e) => {
///             return e.write_errors().into();
///         }
///     };
///
///     // at this point, darling's role is done; use `options.methods` to step
///     // through each of the methods, adding the necessary code for the aspects
///     // that were selected.
/// }
/// ```
///
/// In another crate, this can now be consumed as follows:
///
/// ```rust,ignore
/// #[aspect]
/// impl MyController {
///     #[aspect(trace)]
///     pub fn create(user: UserId, new_resource: Resource) -> Result<ResourceId, MyCreateError> {
///         // elided
///     }
///
///     pub fn get(user: UserId, id: ResourceId) -> Result<ResourceId, MyGetError> {
///         // elided
///     }
///
///     #[aspect(trace, role = "owner")]
///     pub fn delete(user: UserId, id: ResourceId) -> Result<(), MyDeleteError> {
///         // elided
///     }
/// }
/// ```
pub trait FromAttributes: Sized {
    /// Create an instance by parsing a list of attributes.
    ///
    /// By convention, `FromAttributes` implementations should merge item
    /// declarations across attributes, so that the following forms are
    /// equivalent:
    ///
    /// ```rust,ignore
    /// #[derive(Serialize)]
    /// #[serde(rename_all = "camel_case")]
    /// #[serde(borrow)]
    /// pub struct SplitExample {}
    ///
    /// #[derive(Serialize)]
    /// #[serde(borrow, rename_all = "camel_case")]
    /// pub struct JoinedExample {}
    /// ```
    fn from_attributes(attrs: &[Attribute]) -> Result<Self>;
}
