use quote::{Tokens, ToTokens};

/// Declares the local variable into which errors will be accumulated.
pub struct ErrorDeclaration {
    __hidden: ()
}

impl ErrorDeclaration {
    pub fn new() -> Self {
        ErrorDeclaration {
            __hidden: ()
        }
    }
}

impl ToTokens for ErrorDeclaration {
    fn to_tokens(&self, tokens: &mut Tokens) {
        tokens.append(quote! {
            let mut __errors = Vec::new();
        })
    }
}

/// Returns early if attribute or body parsing has caused any errors.
pub struct ErrorCheck {
    __hidden: ()
}

impl ErrorCheck {
    pub fn new() -> Self {
        ErrorCheck {
            __hidden: ()
        }
    }
}

impl ToTokens for ErrorCheck {
    fn to_tokens(&self, tokens: &mut Tokens) {
        tokens.append(quote! {
            if !__errors.is_empty() {
                return ::darling::export::Err(::darling::Error::multiple(__errors));
            }
        })
    }
}