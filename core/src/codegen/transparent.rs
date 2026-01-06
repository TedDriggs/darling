//! Struct is "transparent" (delegates impl to inner field) when there
//! is just a single field. If this field is unnamed, the struct is transparent by default

use proc_macro2::Span;
use syn::{Index, Member};

use crate::{
    ast::{Fields, Style},
    codegen::Field,
};

/// If this struct is "transparent", get the member + inner field
pub fn extract_transparent<'a, 'b>(
    fields: &'a Fields<Field<'b>>,
    is_transparent: bool,
) -> Option<(Member, &'a Field<'b>)> {
    if fields.len() != 1 {
        return None;
    }
    let field = fields
        .iter()
        .next()
        .expect("returned earlier on `.len() != 1`");

    match fields.style {
        Style::Tuple => Some((
            Member::Unnamed(Index {
                index: 0,
                span: Span::call_site(),
            }),
            field,
        )),
        Style::Struct if is_transparent => Some((Member::Named(field.ident.clone()), field)),
        _ => None,
    }
}
