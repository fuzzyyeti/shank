use std::convert::TryFrom;
use std::fmt::Error;

use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, Attribute, Error as ParseError, Ident, Lit, Meta,
    MetaList, MetaNameValue, NestedMeta, Result as ParseResult, Token,
};

const IX_DISCRIMINANT : &str = "discriminant";

#[derive(Debug, PartialEq, Eq)]
pub enum InstructionDiscriminant {
    IncrementDiscriminant { discriminant: u8 },
    ArrayDiscriminant { discriminant: Vec<u8> },
    None,
}


impl InstructionDiscriminant {
    pub(crate) fn is_discriminant_attr(attr: &Attribute) -> Option<&Attribute> {
        match attr
            .path
            .get_ident()
            .map(|x| x.to_string().as_str() == IX_DISCRIMINANT)
        {
            Some(true) => Some(attr),
            _ => None,
        }
    }

    pub fn from_discriminant_attr(
        attr: &Attribute,
    ) -> ParseResult<InstructionDiscriminant> {
        let meta = &attr.parse_meta()?;

        let mut index: Option<u8> = None;
        match meta {
            Meta::List(MetaList { nested, .. }) => {

                let numbers: Result<Vec<u8>,_> = nested.iter()
                    .map(|x| {
                        match x {
                            NestedMeta::Lit(Lit::Int(idx)) => {
                                idx.base10_parse::<u8>()
                            }
                            _ => {
                                Err(ParseError::new_spanned(
                                    x,
                                    "Must be able to parse discriminant numbers into u8",
                                ))
                            }

                        }
                    }).collect();

                match numbers {
                    Ok(numbers) => {
                        if numbers.len() == 1 {
                            return Ok(Self::IncrementDiscriminant { discriminant: numbers[0]})
                        } else if numbers.len() == 8 {
                            return Ok(Self::ArrayDiscriminant { discriminant: numbers })
                        } else {
                            return Err(ParseError::new_spanned(nested,"#[discriminant] attr requires u8 or 8 comma separated u8s"));
                            }
                        }
                    Err(err) => {
                        return Err(err);
                    }
                }
            }
            Meta::Path(_) | Meta::NameValue(_) => Err(ParseError::new_spanned(
                attr,
                "#[account] attr requires list of arguments",
            )),
        }
    }
}

impl TryFrom<&[Attribute]> for InstructionDiscriminant {
    type Error = ParseError;

    fn try_from(attrs: &[Attribute]) -> ParseResult<Self> {
        let discriminant= attrs
            .iter()
            .filter_map(InstructionDiscriminant::is_discriminant_attr)
            .map(InstructionDiscriminant::from_discriminant_attr).next();
        match discriminant {
            Some(Ok(discriminant)) => Ok(discriminant),
            Some(Err(err)) => Err(err),
            None => Ok(Self::None)
        }
    }
}
