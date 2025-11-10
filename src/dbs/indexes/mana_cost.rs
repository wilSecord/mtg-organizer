use tree::tree_traits::{MultidimensionalKey, MultidimensionalParent};

use crate::{
    data_model::card,
    dbs::indexes::{helpers::make_index_types, mana_cost::ManaCostCount::Query},
};

make_index_types! {
    key ManaCostCount {
        num_white: usize,
        num_blue: usize,
        num_black: usize,
        num_red: usize,
        num_green: usize,
        num_colorless: usize,

        num_generic: usize,

        num_any_phyrexian: usize,
        num_any_split_generic: usize,
        num_any_color_split: usize,

        num_variables_used: usize,

        num_odd_edge_case_symbols: usize
    }
}

impl ManaCostCount::Key {
    pub fn new(cost: &card::ManaCost) -> Self {
        let mut slf = Self::smallest_key_in(&ManaCostCount::Query::UNIVERSE);

        for symbol in &cost.0 {
            match symbol {
                card::ManaSymbol::Variable(_) => {
                    slf.num_variables_used += 1;
                }
                card::ManaSymbol::GenericNumber(n) => {
                    slf.num_generic += *n;
                }
                card::ManaSymbol::LandDrop
                | card::ManaSymbol::Legendary
                | card::ManaSymbol::HalfWhite
                | card::ManaSymbol::Snow => {
                    //no need to treat these differently.
                    //these are edge cases anyway, so they're not queryable specifically
                    //in the rapid index.
                    slf.num_odd_edge_case_symbols += 1;
                }
                card::ManaSymbol::OneMillionGenericMana => {
                    slf.num_generic += 1_000_000;
                }
                card::ManaSymbol::ConventionalColored {
                    phyrexian,
                    split_two_generic,
                    color,
                    split_color,
                } => {
                    slf.num_any_phyrexian += (*phyrexian) as usize;
                    slf.num_any_split_generic += (*split_two_generic) as usize;
                    slf.num_any_color_split += split_color.is_some() as usize;

                    match color {
                        card::Color::White => slf.num_white += 1,
                        card::Color::Blue => slf.num_blue += 1,
                        card::Color::Black => slf.num_black += 1,
                        card::Color::Red => slf.num_red += 1,
                        card::Color::Green => slf.num_green += 1,
                        card::Color::Colorless => slf.num_colorless += 1,
                    }
                }
            }
        }

        debug_assert!(slf.is_contained_in(&ManaCostCount::Query::UNIVERSE));

        slf
    }
}
