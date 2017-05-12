use syn;

use {FromField, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariantData<F> {
    Tuple(Vec<F>),
    Struct(Vec<F>),
    Unit,
}

impl<F> VariantData<F> {
    pub fn empty_from(src: &syn::VariantData) -> Self {
        match *src {
            syn::VariantData::Struct(_) => VariantData::Struct(vec![]),
            syn::VariantData::Tuple(_) => VariantData::Tuple(vec![]),
            syn::VariantData::Unit => VariantData::Unit,
        }
    }
} 

impl<F: FromField> VariantData<F> {
    pub fn from(vd: &syn::VariantData) -> Result<Self> {
        match *vd {
            syn::VariantData::Unit => Ok(VariantData::Unit),
            syn::VariantData::Tuple(ref fields) => {
                let mut f = Vec::with_capacity(fields.len());
                for field in fields {
                    f.push(FromField::from_field(field)?);
                }

                Ok(VariantData::Tuple(f))
            },
            syn::VariantData::Struct(ref fields) => {
                let mut f = Vec::with_capacity(fields.len());
                for field in fields {
                    f.push(FromField::from_field(field)?);
                }

                Ok(VariantData::Struct(f))
            }
        }
    }
}