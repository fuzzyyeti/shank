mod account_attrs;
mod extract_instructions;
#[allow(clippy::module_inception)]
mod instruction;
mod strategy_attrs;

mod discriminant_attrs;

pub use account_attrs::*;
pub use extract_instructions::*;
pub use instruction::*;
pub use strategy_attrs::*;
pub use discriminant_attrs::*;

#[cfg(test)]
mod account_attrs_test;
#[cfg(test)]
mod instruction_test;
#[cfg(test)]
mod strategy_attrs_test;
#[cfg(test)]
mod discriminant_attrs;
