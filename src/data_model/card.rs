use std::num::NonZero;

struct Card {
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

enum Rarity {
    Common,
    Mythic,
    Rare,
    Special,
    Uncommon,
}

struct ColorCombination {
    white: bool,
    blue: bool,
    red: bool,
    green: bool,
    black: bool,
    wtf: Option<NonOriginalColor>, //this is for cards like Avatar of Me
}

struct NonOriginalColor {
    hex: [u8; 3],
    name: String,
}

struct NormalManaCost {
    any: usize,
    white: usize,
    blue: usize,
    red: usize,
    green: usize,
    black: usize,
}

enum ManaCost {
    Normal(NormalManaCost),
    Complicated {
        normal_component: NormalManaCost,
        colorless: usize,
        variables: Vec<char>,
        symbol_level_info: Vec<ManaSymbol>,
    },
}

struct ManaSymbol {
    phyrexian: bool,
    color: ColorCombination,
    uncolored_number: Option<NonZero<usize>>,
}
