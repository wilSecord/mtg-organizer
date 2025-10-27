


//General idea of this module is that it provides wrapper structs that represent different MTG types while working as keys for DBTrees. 
//Almost all of these work by converting to/from an unsigned integer of one type or another.

pub mod string;

//general helpers for index wrappers
mod helpers;