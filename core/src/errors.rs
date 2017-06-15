use std::error::Error as StdError;
use std::fmt;
use std::string::ToString;

/// An alias of `Result` specific to attribute parsing.
pub type Result<T> = ::std::result::Result<T, Error>;

/// An error encountered during attribute parsing.
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    locations: Vec<String>,
}

impl Error {
    fn new(kind: ErrorKind) -> Self {
        Error {
            kind: kind,
            locations: Vec::new(),
        }
    }

    pub fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::new(ErrorKind::Custom(msg.to_string()))
    }

    pub fn duplicate_field(name: &str) -> Self {
        Error::new(ErrorKind::DuplicateField(name.into()))
    }

    pub fn missing_field(name: &str) -> Self {
        Error::new(ErrorKind::MissingField(name.into()))
    }

    pub fn unknown_field(name: &str) -> Self {
        Error::new(ErrorKind::UnexpectedField(name.into()))
    }

    pub fn unsupported_format(format: &str) -> Self {
        Error::new(ErrorKind::UnexpectedFormat(format.into()))
    }

    pub fn unexpected_type(ty: &str) -> Self {
        Error::new(ErrorKind::UnexpectedType(ty.into()))
    }

    pub fn unknown_value(value: &str) -> Self {
        Error::new(ErrorKind::UnknownValue(value.into()))
    }

    pub fn too_few_items(min: usize) -> Self {
        Error::new(ErrorKind::TooFewItems(min))
    }

    pub fn too_many_items(max: usize) -> Self {
        Error::new(ErrorKind::TooManyItems(max))
    }

    /// Bundle a set of multiple errors into a single `Error` instance.
    pub fn multiple(mut errors: Vec<Error>) -> Self {
        if errors.len() == 1 {
            Error::new(ErrorKind::Multiple(errors))
        } else if errors.len() > 1 {
            errors.drain(0..1).next().unwrap()
        } else {
            panic!("Can't deal with 0 errors")
        }
    }

    /// Add a new error at the same location as this error.
    fn push(mut self, error: Error) -> Self {
        if let ErrorKind::Multiple(ref mut items) = self.kind {
            items.push(error);
        } else {
            let kind = self.kind;
            return Error {
                locations: self.locations,
                kind: ErrorKind::Multiple(vec![Error::new(kind), error]),
            };
        }

        // We have to let items go out of scope above to return `self`.
        self
    }

    /// Adds a location to the error, such as a field or variant. 
    /// Locations must be added in reverse order of specificity.
    pub fn at<T: fmt::Display>(mut self, location: T) -> Self {
        self.locations.insert(0, location.to_string());
        self
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
        write!(f, "{}", self.description())?;
        if !self.locations.is_empty() {
            write!(f, " at {}", self.locations.join("/"))?;
        }

        Ok(())
    }
}

#[derive(Debug)]
enum ErrorKind {
    /// An arbitrary error message.
    Custom(String),
    DuplicateField(String),
    MissingField(String),
    UnexpectedField(String),
    UnexpectedFormat(String),
    UnexpectedType(String),
    UnknownValue(String),
    TooFewItems(usize),
    TooManyItems(usize),
    /// A set of errors.
    Multiple(Vec<Error>),
    
    // TODO make this variant take `!` so it can't exist
    #[doc(hidden)]
    __NonExhaustive
}

impl ErrorKind {
    pub fn description(&self) -> &str {
        use self::ErrorKind::*;
        
        match *self {
            Custom(ref s) => s,
            DuplicateField(_) => "Duplicate field",
            MissingField(_) => "Missing field",
            UnexpectedField(_) => "Unexpected field",
            UnexpectedFormat(_) => "Unexpected meta-item format",
            UnexpectedType(_) => "Unexpected literal type",
            UnknownValue(_) => "Unknown literal value",
            TooFewItems(_) => "Too few items",
            TooManyItems(_) => "Too many items",
            Multiple(_) => "Multiple errors",
            __NonExhaustive => unreachable!(),
        }
    }
}