use std::{
    num::{IntErrorKind, NonZero, NonZeroUsize, ParseIntError},
    str::FromStr,
};

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
#[derive(Debug, Clone)]
pub struct PhysicalCard {
    pub card: CardRef,
    pub duplicates: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Card {
    pub name: String,
    pub mana_cost: ManaCost,
    /// This is multiplied by 4 because 
    /// of cards with non-integer mana values.
    /// Currently, only cards with .5 mana values
    /// exist, but tokens can be created with .25
    pub mana_value_times_4: usize,
    pub color: ColorCombination,
    pub color_id: ColorCombination,
    pub super_types: Vec<Supertype>,
    pub types: Vec<String>,
    pub subtypes: Vec<String>,
    pub rarity: Rarity,
    pub oracle_text: String,
    pub power: CardDynamicNumber,
    pub toughness: CardDynamicNumber,
    pub loyalty: CardDynamicNumber,
    pub defense: usize,
    pub sets_released: Vec<String>,
    pub game_changer: bool,
}

///
/// Represents some non-negative integer on a MtG card which
/// can be a set value or can be controlled by some
/// manner of game state (e.g. Plague Rats)
/// Even if the value is technically fixed, if it's infinite or
/// negative then it will be treated as dynamic. If it's set, then
/// it has to be <=(USIZE::MAX - 1).
///
/// Represented in source by an Option<NonZeroUsize> for niche optimization.
/// None represents a dynamic number; Some(n) represents the number (n - 1).
#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub struct CardDynamicNumber(Option<NonZeroUsize>);
impl CardDynamicNumber {
    pub fn as_repr_usize(&self) -> usize {
        match self.0 {
            Some(n) => n.get(),
            None => 0,
        }
    }
    pub fn from_repr_usize(u: usize) -> Self {
        match u {
            0 => Self(None),
            _ => Self(Some(NonZeroUsize::new(u).unwrap())),
        }
    }
}

impl Default for CardDynamicNumber {
    fn default() -> Self {
        Self(None)
    }
}

impl FromStr for CardDynamicNumber {
    type Err = <usize as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "*" | "*+1" | "2+*" | "7-*" | "1+*" | "?" | "X" | "1d4+1" | "∞" | "-1" | "-0"
            | "1.5" | "3.5" | ".5" | "2.5" | "*²" => Ok(Self(None)),
            s => match s.parse::<usize>() {
                Ok(v) => Ok(Self(Some(NonZeroUsize::new(v + 1).unwrap()))),
                Err(e) => Err(e),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Supertype {
    Basic,
    Legendary,
    Ongoing,
    Snow,
    World,
    Elite,
    Host,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Mythic,
    Special,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct ColorCombination {
    pub white: bool,
    pub blue: bool,
    pub black: bool,
    pub red: bool,
    pub green: bool,
    pub colorless: bool,
}

#[macro_export]
macro_rules! color_combo {
    ( $($letter:ident )* ) => {
        {
            let mut f = ColorCombination::default();
            $( $crate::color_combo!((letter) $letter, f); )*
            f
        }
    };
    ((letter) w, $f:expr) => { $f.white = true; };
    ((letter) u, $f:expr) => { $f.blue = true; };
    ((letter) b, $f:expr) => { $f.black = true; };
    ((letter) r, $f:expr) => { $f.red = true; };
    ((letter) g, $f:expr) => { $f.green = true; };
    ((letter) c, $f:expr) => { $f.colorless = true; };
}

#[derive(Debug, Clone, Copy, Hash)]
pub enum NormalManaSymbol {
    White,
    Blue,
    Red,
    Green,
    Black,
    Snow, // Check with chloe make sure this is okay
    Colorless,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ManaCost(pub Vec<ManaSymbol>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Blue,
    Red,
    Green,
    Black,
    Colorless,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ManaVariable {
    X,
    Y,
    Z,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ManaSymbol {
    Variable(ManaVariable),
    GenericNumber(usize),
    Snow,
    HalfWhite,
    ConventionalColored {
        phyrexian: bool,
        split_two_generic: bool,
        color: Color,
        split_color: Option<Color>,
    },
    LandDrop,
    Legendary,
    OneMillionGenericMana,
}
