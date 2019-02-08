//! Types for working with darling errors and results.

use proc_macro2::{Span, TokenStream};
use std::error::Error as StdError;
use std::fmt;
use std::iter::{self, Iterator};
use std::string::ToString;
use std::vec;
use syn::spanned::Spanned;
use syn::{Lit, LitStr};

/// An alias of `Result` specific to attribute parsing.
pub type Result<T> = ::std::result::Result<T, Error>;

/// An error encountered during attribute parsing.
///
/// Given that most errors darling encounters represent code bugs in dependent crates,
/// the internal structure of the error is deliberately opaque.
#[derive(Debug)]
#[cfg_attr(test, derive(Clone))]
pub struct Error {
    kind: ErrorKind,
    locations: Vec<String>,
    /// The span to highlight in the emitted diagnostic.
    span: Option<Span>,
}

/// Error creation functions
impl Error {
    fn new(kind: ErrorKind) -> Self {
        Error {
            kind,
            locations: Vec::new(),
            span: None,
        }
    }

    /// Creates a new error with a custom message.
    pub fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::new(ErrorKind::Custom(msg.to_string()))
    }

    /// Creates a new error for a field that appears twice in the input.
    pub fn duplicate_field(name: &str) -> Self {
        Error::new(ErrorKind::DuplicateField(name.into()))
    }

    /// Creates a new error for a non-optional field that does not appear in the input.
    pub fn missing_field(name: &str) -> Self {
        Error::new(ErrorKind::MissingField(name.into()))
    }

    /// Creates a new error for a field name that appears in the input but does not correspond
    /// to a known field.
    pub fn unknown_field(name: &str) -> Self {
        Error::new(ErrorKind::UnknownField(name.into()))
    }

    /// Creates a new error for a field name that appears in the input but does not correspond to
    /// a known attribute. The second argument is the list of known attributes; if a similar name
    /// is found that will be shown in the emitted error message.
    pub fn unknown_field_with_alts<'a, T, I>(field: &str, alternates: I) -> Self
    where
        T: AsRef<str> + 'a,
        I: IntoIterator<Item = &'a T>,
    {
        let did_you_mean = did_you_mean(field, alternates);
        Error::new(ErrorKind::UnknownField(ErrorUnknownField::new(
            field,
            did_you_mean,
        )))
    }

    /// Creates a new error for a struct or variant that does not adhere to the supported shape.
    pub fn unsupported_shape(shape: &str) -> Self {
        Error::new(ErrorKind::UnsupportedShape(shape.into()))
    }

    pub fn unsupported_format(format: &str) -> Self {
        Error::new(ErrorKind::UnexpectedFormat(format.into()))
    }

    /// Creates a new error for a field which has an unexpected literal type.
    pub fn unexpected_type(ty: &str) -> Self {
        Error::new(ErrorKind::UnexpectedType(ty.into()))
    }

    /// Creates a new error for a field which has an unexpected literal type. This will automatically
    /// extract the literal type name from the passed-in `Lit` and set the span to encompass only the
    /// literal value.
    ///
    /// # Usage
    /// This is most frequently used in overrides of the `FromMeta::from_value` method.
    ///
    /// ```rust
    /// # // pretend darling_core is darling so the doc example looks correct.
    /// # extern crate darling_core as darling;
    /// # extern crate syn;
    ///
    /// use darling::{FromMeta, Error, Result};
    /// use syn::{Lit, LitStr};
    ///
    /// pub struct Foo(String);
    ///
    /// impl FromMeta for Foo {
    ///     fn from_value(value: &Lit) -> Result<Self> {
    ///         if let Lit::Str(ref lit_str) = value {
    ///             Ok(Foo(lit_str.value()))
    ///         } else {
    ///             Err(Error::unexpected_lit_type(value))
    ///         }
    ///     }
    /// }
    ///
    /// # fn main() {}
    /// ```
    pub fn unexpected_lit_type(lit: &Lit) -> Self {
        Error::unexpected_type(match *lit {
            Lit::Str(_) => "string",
            Lit::ByteStr(_) => "byte string",
            Lit::Byte(_) => "byte",
            Lit::Char(_) => "char",
            Lit::Int(_) => "int",
            Lit::Float(_) => "float",
            Lit::Bool(_) => "bool",
            Lit::Verbatim(_) => "verbatim",
        })
        .with_span(lit)
    }

    /// Creates a new error for a value which doesn't match a set of expected literals.
    pub fn unknown_value(value: &str) -> Self {
        Error::new(ErrorKind::UnknownValue(value.into()))
    }

    /// Creates a new error for a list which did not get enough items to proceed.
    pub fn too_few_items(min: usize) -> Self {
        Error::new(ErrorKind::TooFewItems(min))
    }

    /// Creates a new error when a list got more items than it supports. The `max` argument
    /// is the largest number of items the receiver could accept.
    pub fn too_many_items(max: usize) -> Self {
        Error::new(ErrorKind::TooManyItems(max))
    }

    /// Bundle a set of multiple errors into a single `Error` instance.
    ///
    /// # Panics
    /// This function will panic if `errors.is_empty() == true`.
    pub fn multiple(mut errors: Vec<Error>) -> Self {
        if errors.len() > 1 {
            Error::new(ErrorKind::Multiple(errors))
        } else if errors.len() == 1 {
            errors
                .pop()
                .expect("Error array of length 1 has a first item")
        } else {
            panic!("Can't deal with 0 errors")
        }
    }
}

impl Error {
    /// Create a new error about a literal string that doesn't match a set of known
    /// or permissible values. This function can be made public if the API proves useful
    /// beyond impls for `syn` types.
    pub(crate) fn unknown_lit_str_value(value: &LitStr) -> Self {
        Error::unknown_value(&value.value()).with_span(value)
    }
}

/// Error instance methods
impl Error {
    /// Check if this error is associated with a span in the token stream.
    pub fn has_span(&self) -> bool {
        self.span.is_some()
    }

    /// Override the source code location of this error with a new one.
    #[deprecated(
        since = "0.8.3",
        note = "Callers should not broaden error spans. Use with_span instead."
    )]
    pub fn set_span(&mut self, span: Span) {
        self.span = Some(span)
    }

    /// Tie a span to the error if none is already present. This is used in `darling::FromMeta`
    /// and other traits to attach errors to the most specific possible location in the input
    /// source code.
    ///
    /// All `darling`-built impls, either from the crate or from the proc macro, will call this
    /// when appropriate during parsing, so it should not be necessary to call this unless you have
    /// overridden:
    ///
    /// * `FromMeta::from_meta`
    /// * `FromMeta::from_nested_meta`
    /// * `FromMeta::from_value`
    pub fn with_span<T: Spanned>(mut self, node: &T) -> Self {
        if !self.has_span() {
            self.span = Some(node.span());
        }

        self
    }

    /// Recursively converts a tree of errors to a flattened list.
    pub fn flatten(self) -> Self {
        Error::multiple(self.into_vec())
    }

    fn into_vec(self) -> Vec<Self> {
        if let ErrorKind::Multiple(errors) = self.kind {
            let mut flat = Vec::new();
            for error in errors {
                flat.extend(error.prepend_at(self.locations.clone()).into_vec());
            }

            flat
        } else {
            vec![self]
        }
    }

    /// Adds a location to the error, such as a field or variant.
    /// Locations must be added in reverse order of specificity.
    pub fn at<T: fmt::Display>(mut self, location: T) -> Self {
        self.locations.insert(0, location.to_string());
        self
    }

    /// Gets the number of individual errors in this error.
    ///
    /// This function never returns `0`, as it's impossible to construct
    /// a multi-error from an empty `Vec`.
    pub fn len(&self) -> usize {
        self.kind.len()
    }

    /// Adds a location chain to the head of the error's existing locations.
    fn prepend_at(mut self, mut locations: Vec<String>) -> Self {
        if !locations.is_empty() {
            locations.extend(self.locations);
            self.locations = locations;
        }

        self
    }

    /// Gets the location slice.
    #[cfg(test)]
    pub(crate) fn location(&self) -> Vec<&str> {
        self.locations.iter().map(|i| i.as_str()).collect()
    }

    /// Write this error and any children as compile errors into a `TokenStream` to
    /// be returned by the proc-macro.
    ///
    /// The behavior of this method will be slightly different if the `diagnostics` feature
    /// is enabled: In that case, the diagnostics will be emitted immediately by this call,
    /// and an empty `TokenStream` will be returned.
    ///
    /// Return these tokens unmodified to avoid disturbing the attached span information.
    ///
    /// # Usage
    /// ```rust,ignore
    /// // in your proc-macro function
    /// let opts = match MyOptions::from_derive_input(&ast) {
    ///     Ok(val) => val,
    ///     Err(err) => {
    ///         return err.write_errors();
    ///     }
    /// }
    /// ```
    pub fn write_errors(self) -> TokenStream {
        #[cfg(feature = "diagnostics")]
        {
            self.emit();
            quote!()
        }

        #[cfg(not(feature = "diagnostics"))]
        {
            self.flatten()
                .into_iter()
                .map(|e| e.single_to_syn_error().to_compile_error())
                .collect()
        }
    }

    #[cfg(not(feature = "diagnostics"))]
    fn single_to_syn_error(self) -> ::syn::Error {
        match self.span {
            Some(span) => ::syn::Error::new(span, self.kind),
            None => ::syn::Error::new(Span::call_site(), self),
        }
    }

    #[cfg(feature = "diagnostics")]
    fn single_to_diagnostic(self) -> ::proc_macro::Diagnostic {
        use proc_macro::{Diagnostic, Level};

        // Delegate to dedicated error formatters when applicable.
        //
        // If span information is available, don't include the error property path
        // since it's redundant and not consistent with native compiler diagnostics.
        match self.kind {
            ErrorKind::UnknownField(euf) => euf.to_diagnostic(self.span),
            kind => match self.span {
                Some(span) => span.unwrap().error(kind.to_string()),
                None => Diagnostic::new(Level::Error, self.to_string()),
            }
        }
    }

    /// Transform this error and its children into a list of compiler diagnostics
    /// and emit them. If the `Error` has associated span information, the diagnostics
    /// will identify the correct location in source code automatically.
    ///
    /// # Stability
    /// This is only available on `nightly` until the compiler `proc_macro_diagnostic`
    /// feature stabilizes. Until then, it may break at any time.
    #[cfg(feature = "diagnostics")]
    pub fn emit(self) {
        for error in self.flatten() {
            error.single_to_diagnostic().emit()
        }
    }

    /// Transform the error into a compiler diagnostic and - if the diagnostic points to
    /// a specific code location - add a spanned help child diagnostic that points to the
    /// parent derived trait.
    ///
    /// This is experimental and therefore not exposed outside the crate.
    #[cfg(feature = "diagnostics")]
    #[allow(dead_code)]
    fn emit_with_macro_help_span(self) {
        use proc_macro::Diagnostic;

        for error in self.flatten() {
            let needs_help = error.has_span();
            let diagnostic = error.single_to_diagnostic();
            Diagnostic::emit(if needs_help {
                diagnostic.span_help(
                    Span::call_site().unwrap(),
                    "Encountered as part of this derive-mode-macro",
                )
            } else {
                diagnostic
            })
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        &self.kind.description()
    }

    fn cause(&self) -> Option<&StdError> {
        None
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.kind)?;
        if !self.locations.is_empty() {
            write!(f, " at {}", self.locations.join("/"))?;
        }

        Ok(())
    }
}

// Don't want to publicly commit to Error supporting equality yet, but
// not having it makes testing very difficult. Note that spans are not
// considered for equality since that would break testing in most cases.
#[cfg(test)]
impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.locations == other.locations
    }
}

#[cfg(test)]
impl Eq for Error {}

impl IntoIterator for Error {
    type Item = Error;
    type IntoIter = IntoIter;

    fn into_iter(self) -> IntoIter {
        if let ErrorKind::Multiple(errors) = self.kind {
            IntoIter {
                inner: IntoIterEnum::Multiple(errors.into_iter()),
            }
        } else {
            IntoIter {
                inner: IntoIterEnum::Single(iter::once(self)),
            }
        }
    }
}

enum IntoIterEnum {
    Single(iter::Once<Error>),
    Multiple(vec::IntoIter<Error>),
}

impl Iterator for IntoIterEnum {
    type Item = Error;

    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            IntoIterEnum::Single(ref mut content) => content.next(),
            IntoIterEnum::Multiple(ref mut content) => content.next(),
        }
    }
}

/// An iterator that moves out of an `Error`.
pub struct IntoIter {
    inner: IntoIterEnum,
}

impl Iterator for IntoIter {
    type Item = Error;

    fn next(&mut self) -> Option<Error> {
        self.inner.next()
    }
}

type DeriveInputShape = String;
type FieldName = String;
type MetaFormat = String;

#[derive(Debug)]
// Don't want to publicly commit to ErrorKind supporting equality yet, but
// not having it makes testing very difficult.
#[cfg_attr(test, derive(Clone, PartialEq, Eq))]
enum ErrorKind {
    /// An arbitrary error message.
    Custom(String),
    DuplicateField(FieldName),
    MissingField(FieldName),
    UnsupportedShape(DeriveInputShape),
    UnknownField(ErrorUnknownField),
    UnexpectedFormat(MetaFormat),
    UnexpectedType(String),
    UnknownValue(String),
    TooFewItems(usize),
    TooManyItems(usize),
    /// A set of errors.
    Multiple(Vec<Error>),

    // TODO make this variant take `!` so it can't exist
    #[doc(hidden)]
    __NonExhaustive,
}

impl ErrorKind {
    pub fn description(&self) -> &str {
        use self::ErrorKind::*;

        match *self {
            Custom(ref s) => s,
            DuplicateField(_) => "Duplicate field",
            MissingField(_) => "Missing field",
            UnknownField(_) => "Unexpected field",
            UnsupportedShape(_) => "Unsupported shape",
            UnexpectedFormat(_) => "Unexpected meta-item format",
            UnexpectedType(_) => "Unexpected literal type",
            UnknownValue(_) => "Unknown literal value",
            TooFewItems(_) => "Too few items",
            TooManyItems(_) => "Too many items",
            Multiple(_) => "Multiple errors",
            __NonExhaustive => unreachable!(),
        }
    }

    /// Deeply counts the number of errors this item represents.
    pub fn len(&self) -> usize {
        if let ErrorKind::Multiple(ref items) = *self {
            items.iter().map(Error::len).sum()
        } else {
            1
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ErrorKind::*;

        match *self {
            Custom(ref s) => s.fmt(f),
            DuplicateField(ref field) => write!(f, "Duplicate field `{}`", field),
            MissingField(ref field) => write!(f, "Missing field `{}`", field),
            UnknownField(ref field) => field.fmt(f),
            UnsupportedShape(ref shape) => write!(f, "Unsupported shape `{}`", shape),
            UnexpectedFormat(ref format) => write!(f, "Unexpected meta-item format `{}`", format),
            UnexpectedType(ref ty) => write!(f, "Unexpected literal type `{}`", ty),
            UnknownValue(ref val) => write!(f, "Unknown literal value `{}`", val),
            TooFewItems(ref min) => write!(f, "Too few items: Expected at least {}", min),
            TooManyItems(ref max) => write!(f, "Too many items: Expected no more than {}", max),
            Multiple(ref items) if items.len() == 1 => items[0].fmt(f),
            Multiple(ref items) => {
                write!(f, "Multiple errors: (")?;
                let mut first = true;
                for item in items {
                    if !first {
                        write!(f, ", ")?;
                    } else {
                        first = false;
                    }

                    item.fmt(f)?;
                }

                write!(f, ")")
            }
            __NonExhaustive => unreachable!(),
        }
    }
}

impl From<ErrorUnknownField> for ErrorKind {
    fn from(err: ErrorUnknownField) -> Self {
        ErrorKind::UnknownField(err)
    }
}

/// An error for an unknown field, with a possible "did-you-mean" suggestion to get
/// the user back on the right track.
#[derive(Debug)]
// Don't want to publicly commit to ErrorKind supporting equality yet, but
// not having it makes testing very difficult.
#[cfg_attr(test, derive(Clone, PartialEq, Eq))]
struct ErrorUnknownField {
    name: String,
    did_you_mean: Option<String>,
}

impl ErrorUnknownField {
    pub fn new<I: Into<String>>(name: I, did_you_mean: Option<String>) -> Self {
        ErrorUnknownField {
            name: name.into(),
            did_you_mean,
        }
    }

    #[cfg(feature = "diagnostics")]
    pub fn to_diagnostic(self, span: Option<Span>) -> ::proc_macro::Diagnostic {
        let base = span.unwrap().unwrap().error(self.top_line());
        match self.did_you_mean {
            Some(alt_name) => base.help(format!("did you mean `{}`?", alt_name)),
            None => base,
        }
    }

    #[cfg(feature = "diagnostics")]
    fn top_line(&self) -> String {
        format!("Unknown field: `{}`", self.name)
    }
}

impl From<String> for ErrorUnknownField {
    fn from(name: String) -> Self {
        ErrorUnknownField::new(name, None)
    }
}

impl<'a> From<&'a str> for ErrorUnknownField {
    fn from(name: &'a str) -> Self {
        ErrorUnknownField::new(name, None)
    }
}

impl fmt::Display for ErrorUnknownField {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unknown field: `{}`", self.name)?;

        if let Some(ref did_you_mean) = self.did_you_mean {
            write!(f, ". Did you mean `{}`?", did_you_mean)?;
        }

        Ok(())
    }
}

#[cfg(feature = "suggestions")]
fn did_you_mean<'a, T, I>(field: &str, alternates: I) -> Option<String>
where
    T: AsRef<str> + 'a,
    I: IntoIterator<Item = &'a T>,
{
    let mut candidate: Option<(f64, &str)> = None;
    for pv in alternates {
        let confidence = ::strsim::jaro_winkler(field, pv.as_ref());
        if confidence > 0.8 && (candidate.is_none() || (candidate.as_ref().unwrap().0 < confidence))
        {
            candidate = Some((confidence, pv.as_ref()));
        }
    }
    match candidate {
        None => None,
        Some((_, candidate)) => Some(candidate.into()),
    }
}

#[cfg(not(feature = "suggestions"))]
fn did_you_mean<'a, T, I>(_field: &str, _alternates: I) -> Option<String>
where
    T: AsRef<str> + 'a,
    I: IntoIterator<Item = &'a T>,
{
    None
}

#[cfg(test)]
mod tests {
    use super::Error;

    #[test]
    fn flatten_noop() {
        let err = Error::duplicate_field("hello").at("world");
        assert_eq!(err.clone().flatten(), err);
    }

    #[test]
    fn flatten_simple() {
        let err = Error::multiple(vec![
            Error::unknown_field("hello").at("world"),
            Error::missing_field("hell_no").at("world"),
        ])
        .at("foo")
        .flatten();

        assert!(err.location().is_empty());

        let mut err_iter = err.into_iter();

        let first = err_iter.next();
        assert!(first.is_some());
        assert_eq!(first.unwrap().location(), vec!["foo", "world"]);

        let second = err_iter.next();
        assert!(second.is_some());

        assert_eq!(second.unwrap().location(), vec!["foo", "world"]);

        assert!(err_iter.next().is_none());
    }

    #[test]
    fn len_single() {
        let err = Error::duplicate_field("hello");
        assert_eq!(1, err.len());
    }

    #[test]
    fn len_multiple() {
        let err = Error::multiple(vec![
            Error::duplicate_field("hello"),
            Error::missing_field("hell_no"),
        ]);
        assert_eq!(2, err.len());
    }

    #[test]
    fn len_nested() {
        let err = Error::multiple(vec![
            Error::duplicate_field("hello"),
            Error::multiple(vec![
                Error::duplicate_field("hi"),
                Error::missing_field("bye"),
                Error::multiple(vec![Error::duplicate_field("whatsup")]),
            ]),
        ]);

        assert_eq!(4, err.len());
    }
}
