use std::num::NonZero;

use crate::data_model::oddities::{Stringish, StringishUsize};

///
/// Reference to a specific card, can be as specific as needed or vague to be only set + collector number.
/// Something will be made where code can exchange this for a full `Card`
///
#[derive(Debug, Clone)]
pub struct CardRef {
    pub set: String,
    pub collector_number: StringishUsize,
    pub printing: Option<NonZero<usize>>,
}

///
/// One physical card. Users may have more than one `PhysicalCard` with the same `CardRef` in their collection; this might be
/// implemented differently (i.e. many `PhysicalCards` or one `PhysicalCard` with `duplicates`)
/// depending on how the user choses to arrange their collection.
pub struct PhysicalCard {
    pub card: CardRef,
    pub duplicates: usize,
}

pub struct Card {
    pub name: String,
    pub mana_cost: ManaCost,
    pub mana_value: f64,
    pub color: ColorCombination,
    pub color_id: ColorCombination,
    pub super_types: Vec<Supertype>,
    pub types: Vec<String>,
    pub subtypes: Vec<String>,
    pub rarity: Rarity,
    pub oracle_text: String,
    pub power: usize,
    pub toughness: usize,
    pub loyalty: usize,
    pub defense: usize,
    pub sets_released: Vec<String>,
    pub game_changer: bool,
}

pub enum Supertype {
    Basic,
    Legendary,
    Ongoing,
    Snow,
    World,
    Elite,
    Host
}

pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Mythic,
    Special,
}

pub struct ColorCombination {
    pub white: bool,
    pub blue: bool,
    pub black: bool,
    pub red: bool,
    pub green: bool,
    pub colorless: bool,
}

pub enum NormalManaSymbol {
    White,
    Blue,
    Red,
    Green,
    Black,
    Snow, // Check with chloe make sure this is okay
    Colorless,
}

pub struct NormalManaCost {
    pub symbols: Vec<NormalManaSymbol>,
    pub generic: usize
}

pub enum ManaCost {
    Normal(NormalManaCost),
    Complicated(NormalManaCost, ComplicatedManaCases),
}

pub enum ComplicatedManaCases {
    Variables(Vec<char>),
    SymbolLevel(Vec<ManaSymbol>)
}

pub enum Color {
    White,
    Blue,
    Red,
    Green,
    Black,
    Colorless,
}

pub struct ManaSymbol {
    pub phyrexian: bool,
    pub split_two_generic: bool,    
    pub color: Color,
    pub split_color: Option<Color>,
}
