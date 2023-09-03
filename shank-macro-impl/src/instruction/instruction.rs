use std::{
    collections::HashSet,
    convert::{TryFrom, TryInto},
};
use proc_macro2::Span;
use syn::{Attribute, Error as ParseError, ItemEnum, Result as ParseResult};

use syn::Ident;

use crate::{
    parsed_enum::{ParsedEnum, ParsedEnumVariant},
    parsers::get_derive_attr,
    types::RustType,
    DERIVE_INSTRUCTION_ATTR,
};
use crate::instruction::discriminant_attrs::InstructionDiscriminant;

use super::{
    account_attrs::{InstructionAccount, InstructionAccounts},
    InstructionStrategies, InstructionStrategy,
};

// -----------------
// Instruction
// -----------------
#[derive(Debug)]
pub struct Instruction {
    pub ident: Ident,
    pub variants: Vec<InstructionVariant>,
}

impl Instruction {
    pub fn try_from_item_enum(
        item_enum: &ItemEnum,
        skip_derive_attr_check: bool,
    ) -> ParseResult<Option<Instruction>> {
        if skip_derive_attr_check
            || get_derive_attr(&item_enum.attrs, DERIVE_INSTRUCTION_ATTR)
            .is_some()
        {
            let parsed_enum = ParsedEnum::try_from(item_enum)?;
            Instruction::try_from(&parsed_enum).map(Some)
        } else {
            Ok(None)
        }
    }
}

impl TryFrom<&ParsedEnum> for Option<Instruction> {
    type Error = ParseError;

    fn try_from(parsed_enum: &ParsedEnum) -> ParseResult<Self> {
        match get_derive_attr(&parsed_enum.attrs, DERIVE_INSTRUCTION_ATTR)
            .map(|_| parsed_enum)
        {
            Some(ix_enum) => ix_enum.try_into().map(Some),
            None => Ok(None),
        }
    }
}

impl TryFrom<&ParsedEnum> for Instruction {
    type Error = ParseError;

    fn try_from(parsed_enum: &ParsedEnum) -> ParseResult<Self> {
        let ParsedEnum {
            ident, variants, ..
        } = parsed_enum;

        let variants = variants
            .iter()
            .map(InstructionVariant::try_from)
            .collect::<ParseResult<Vec<InstructionVariant>>>()?;
        Ok(Self {
            ident: ident.clone(),
            variants,
        })
    }
}

#[derive(Debug)]
pub enum InstructionVariantFields {
    Unnamed(Vec<RustType>),
    Named(Vec<(String, RustType)>),
}

// -----------------
// Instruction Variant
// -----------------
#[derive(Debug)]
pub struct InstructionVariant {
    pub ident: Ident,
    pub field_tys: InstructionVariantFields,
    pub accounts: Vec<InstructionAccount>,
    pub strategies: HashSet<InstructionStrategy>,
    pub discriminant: InstructionDiscriminant,
}

impl TryFrom<&ParsedEnumVariant> for InstructionVariant {
    type Error = ParseError;

    fn try_from(variant: &ParsedEnumVariant) -> ParseResult<Self> {
        let ParsedEnumVariant {
            ident,
            fields,
            discriminant,
            attrs,
            ..
        } = variant;


        let field_tys: InstructionVariantFields = if !fields.is_empty() {
            // Determine if the InstructionType is tuple or struct variant
            let field = fields.get(0).unwrap();
            match &field.ident {
                Some(_) => InstructionVariantFields::Named(
                    fields
                        .iter()
                        .map(|x| {
                            (
                                x.ident.as_ref().unwrap().to_string(),
                                x.rust_type.clone(),
                            )
                        })
                        .collect(),
                ),
                None => InstructionVariantFields::Unnamed(
                    fields.iter().map(|x| x.rust_type.clone()).collect(),
                ),
            }
        } else {
            InstructionVariantFields::Unnamed(vec![])
        };

        let attrs_ref: &[Attribute] = attrs.as_ref();
        let accounts: InstructionAccounts = attrs_ref.try_into()?;
        let strategies: InstructionStrategies = attrs_ref.into();
        let explicit_descriminant: InstructionDiscriminant = attrs_ref.try_into()?;
        let final_descriminant;

        match explicit_descriminant {
            InstructionDiscriminant::None => {
                if *discriminant >= u8::MAX as usize {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        format!("Instruction variant discriminants have to be <= u8::MAX ({}), \
                        but the discriminant of variant '{}' is {}",
                                u8::MAX,
                                ident,
                                discriminant)));
                }
                final_descriminant = InstructionDiscriminant::IncrementDiscriminant { discriminant: *discriminant as u8 }
            }
            _ => {
                final_descriminant = explicit_descriminant
            }
        }

        Ok(Self {
            ident: ident.clone(),
            field_tys,
            accounts: accounts.0,
            strategies: strategies.0,
            discriminant: final_descriminant,
        })
    }
}
