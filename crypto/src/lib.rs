// #![cfg_attr(not(feature = "std"), no_std)]

// #[cfg(not(feature = "std"))]
// #[cfg_attr(test, macro_use)]
// #![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

pub mod dsa;
pub mod hash;
pub mod merkle;
pub mod rand;
pub mod utils;

// RE-EXPORTS
// ================================================================================================

pub use winter_math::{fields::f64::BaseElement as Felt, FieldElement, StarkField};

// TYPE ALIASES
// ================================================================================================

/// A group of four field elements in the Miden base field.
pub type Word = [Felt; WORD_SIZE];

// CONSTANTS
// ================================================================================================

/// Number of field elements in a word.
pub const WORD_SIZE: usize = 4;

/// Field element representing ZERO in the Miden base filed.
pub const ZERO: Felt = Felt::ZERO;

/// Field element representing ONE in the Miden base filed.
pub const ONE: Felt = Felt::ONE;

/// Array of field elements representing word of ZEROs in the Miden base field.
pub const EMPTY_WORD: [Felt; 4] = [ZERO; WORD_SIZE];

// TESTS
// ================================================================================================


