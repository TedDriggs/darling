//! Utility types for attribute parsing.

mod body;
mod ident_list;
mod over_ride;
mod variant_data;

pub use self::body::Body;
pub use self::ident_list::IdentList;
pub use self::over_ride::Override;
pub use self::variant_data::VariantData;