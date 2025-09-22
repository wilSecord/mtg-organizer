use std::num::NonZero;

use crate::data_model::oddities::{Stringish, StringishUsize};

///
/// Reference to a specific card, can be as specific as needed or vague to be only set + collector number.
/// Something will be made where code can exchange this for a full `Card`
///
pub struct CardRef {
    set: String,
    collector_number: StringishUsize,
    printing: Option<NonZero<usize>>,
}

///
/// One physical card. Users may have more than one `PhysicalCard` with the same `CardRef` in their collection; this might be
/// implemented differently (i.e. many `PhysicalCards` or one `PhysicalCard` with `duplicates`)
/// depending on how the user choses to arrange their collection.
pub struct PhysicalCard {
    card: CardRef,
    duplicates: usize,
}

pub struct Card {
    name: String,
    mana_cost: ManaCost,
    mana_value: f64,
    color: ColorCombination,
    color_id: ColorCombination,
    super_types: Vec<String>,
    types: Vec<String>,
    rarity: Rarity,
    oracle_text: String,
    power: usize,
    toughness: usize,
    subtypes: Vec<String>,
    loyalty: usize,
    defense: usize,
    sets_released: Vec<String>,
    game_changer: bool,
}

pub enum Rarity {
    Common,
    Mythic,
    Rare,
    Special,
    Uncommon,
}

pub struct ColorCombination {
    white: bool,
    blue: bool,
    red: bool,
    green: bool,
    black: bool,
}

pub struct NormalManaCost {
    any: usize,
    white: usize,
    blue: usize,
    red: usize,
    green: usize,
    black: usize,
    colorless: usize,
}

pub enum ManaCost {
    Normal(NormalManaCost),
    Complicated {
        normal_component: NormalManaCost,
        variables: Vec<char>,
        symbol_level_info: Vec<ManaSymbol>,
    },
}

pub struct ManaSymbol {
    phyrexian: bool,
    color: ColorCombination,
    is_split_generic: bool,
}
