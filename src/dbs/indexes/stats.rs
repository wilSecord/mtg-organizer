use std::{ops::{Add, Sub}, usize};

use tree::tree_traits::{
    Average, MaxValue, MinValue, MultidimensionalKey, MultidimensionalParent, Zero,
};

use crate::{
    data_model::card::{self, Card, CardDynamicNumber},
    dbs::indexes::{helpers::make_index_types, mana_cost::ManaCostCount::Query},
};

make_index_types! {
    key card_stats {
        power: usize,
        toughness: usize,
        loyalty: usize,
        defense: usize,
        game_changer: u8,
        mana_value_quarters: usize,
    }
}

impl card_stats::Key {
    pub fn new(card: &Card) -> Self {
        Self {
            power: card.power.as_repr_usize(),
            toughness: card.toughness.as_repr_usize(),
            loyalty: card.loyalty.as_repr_usize(),
            defense: card.defense + 1,
            game_changer: card.game_changer as u8,
            mana_value_quarters: (card.mana_value * 4.0) as usize,
        }
    }
}