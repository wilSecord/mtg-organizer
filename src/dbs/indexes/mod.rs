


//General idea of this module is that it provides wrapper structs that represent different MTG types while working as keys for DBTrees. 
//Almost all of these work by converting to/from an unsigned integer of one type or another.

pub mod string_lpm;
pub mod string_trigram;
pub mod color_combination;
pub mod mana_cost;
pub mod stats;

mod rarity_supertype;
pub use rarity_supertype::{rarity, supertype};

//general helpers for index wrappers
mod helpers;


