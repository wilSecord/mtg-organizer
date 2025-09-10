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
    game_changer: bool
}

enum Rarity {
    Common,
    Mythic,
    Rare,
    Special,
    Uncommon
}

struct ColorCombination {
    white: bool,
    blue: bool,
    red: bool,
    green: bool,
    black: bool,
}

struct ManaCost {
    variable_cost: bool,
    any: usize,
    white: usize,
    blue: usize,
    red: usize,
    green: usize,
    black: usize
}