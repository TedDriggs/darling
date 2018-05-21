use fnv::FnvHashSet;
use syn::Ident;

pub type IdentSet = FnvHashSet<Ident>;
pub type IdentRefSet<'a> = FnvHashSet<&'a Ident>;
