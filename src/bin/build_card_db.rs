use std::io::{self, BufRead, BufReader};
use std::fs::{File, read_to_string};
use project::data_model::card::{self, Card, Color, ColorCombination, ManaCost, ManaSymbol, Supertype};
use serde_json;

const APPNAME_DIRECTORY: &'static str = "mtg-organizer";

fn parse_color_combination(combo: &str) -> ColorCombination {
    let mut cc = ColorCombination::default();
    for chs in combo.chars() {
        match chs {
            'W' => cc.white = true,
            'U' => cc.blue = true,
            'B' => cc.black = true,
            'R' => cc.red = true,
            'G' => cc.green = true,
            'C' => cc.colorless = true,
            _ => {}
        }
    }

    cc
}

fn parse_card(card: serde_json::Value) -> Card {
    return Card{
        name: card["name"].to_string(),
        mana_cost: parse_mana_cost(card["mana_cost"].as_str().expect("Mana cost should be a string")),
        mana_value: card["mana_value"].as_f64().expect("Bad MV"),
        color: parse_color_combination(card["color"].as_str().expect("Bad Color Combo")),
        color_id: parse_color_combination(card["color_id"].as_str().expect("Bad Color Combo")),
        super_types: card["super_types"].as_str().expect("super_types should be a string").split("/").map(|x| match x {
            "Basic" => Supertype::Basic,
            "Legendary" => Supertype::Legendary,
            "Elite" => Supertype::Elite,
            "Ongoing" => Supertype::Ongoing,
            "Host" => Supertype::Host,
            "World" => Supertype::World,
            "Snow" => Supertype::Snow,
            _ => panic!("Unknown supertype {x}")
        }).collect(),
        rarity: match card["rarity"].as_str().expect("Bad Rarity") {
            "common" => card::Rarity::Common,
            "uncommon" => card::Rarity::Uncommon,
            "rare" => card::Rarity::Rare,
            "mythic" => card::Rarity::Mythic,
            "special" => card::Rarity::Special,
            other => panic!("Unexpected rarity value {other}")
        },
        oracle_text: card["oracle_text"].to_string(),
        power: card["power"].as_u64().expect("Bad Power") as usize,
        toughness: card["toughness"].as_u64().expect("Bad Toughness") as usize,
        types: card["types"].as_str().expect("Bad types").split(", ").map(String::from).collect(),
        subtypes: card["subtypes"].as_str().expect("Bad types").split(", ").map(String::from).collect(),
        loyalty: card["loyalty"].as_u64().expect("Bad loyalty") as usize,
        defense: card["defense"].as_u64().expect("Bad defense") as usize,
        sets_released: card["sets_released"].as_str().expect("Bad types").split(", ").map(String::from).collect(),
        game_changer: card["game_changer"].as_str().expect("Bad game_changer") == "true",
        

    };
    // for (key, value) in card_obj {

    // }
    
}

fn scan_mana_symbol<'a>(buf: &mut &'a str) -> &'a str {
    for (i, ch) in buf.char_indices() {
        //opening bracket will already be handled by parse_mana_cost
        if ch == '}' {
            let (src, new_buf) = buf.split_at(i);
            *buf = new_buf;
            return src;
        }
    }
    return "";
}

fn parse_mana_symbol(src: &str) -> ManaSymbol {
    //rules text: 107.4. The mana symbols are {W}, {U}, {B}, {R}, {G}, and {C}; 
    //              the numerical symbols {0}, {1}, {2}, {3}, {4}, and so on; the 
    //              variable symbol {X}; the hybrid symbols {W/U}, {W/B}, {U/B}, {U/R}, 
    //              {B/R}, {B/G}, {R/G}, {R/W}, {G/W}, and {G/U}; the monocolored hybrid 
    //              symbols {2/W}, {2/U}, {2/B}, {2/R}, {2/G}, {C/W}, {C/U}, {C/B}, {C/R}, 
    //              and {C/G}; the Phyrexian mana symbols {W/P}, {U/P}, {B/P}, {R/P}, and {G/P}; 
    //              the hybrid Phyrexian symbols {W/U/P}, {W/B/P}, {U/B/P}, {U/R/P}, {B/R/P}, {B/G/P}, 
    //              {R/G/P}, {R/W/P}, {G/W/P}, and {G/U/P}; and the snow mana symbol {S}.

    if src.chars().all(|x| x.is_ascii_digit()) {
        return ManaSymbol::GenericNumber(src.parse().unwrap());
    }

    match src {
        "S" => return ManaSymbol::Snow,
        "X" => return ManaSymbol::Variable(card::ManaVariable::X),
        "Y" => return ManaSymbol::Variable(card::ManaVariable::Y),
        "Z" => return ManaSymbol::Variable(card::ManaVariable::Z),
        _ => {}
    }

    //having completed that, it's definitely going to be a conventional coloured 
    // mana symbol of some kind.

    let mut split_two_generic = false;
    let mut phyrexian = false;

    let mut colors = Vec::with_capacity(2);

    for spec in src.split("/") {
        match spec {
            "P" => phyrexian = true,
            "2" => split_two_generic = true,
            c => colors.push(parse_color(c.chars().next().expect("Empty colour name"))),
        }
    }

    assert!(colors.len() <= 2);

    if colors.len() == 2 {
        let split_color = colors.pop();
        let color = colors.pop().unwrap();
        return ManaSymbol::ConventionalColored { phyrexian, split_two_generic, color, split_color };
    } else if colors.len() == 1 {
        let split_color = None;
        let color = colors.pop().unwrap();
        return ManaSymbol::ConventionalColored { phyrexian, split_two_generic, color, split_color };
    } else {
        panic!("Bad mana symbol {{{src}}}")
    }
}

fn parse_color(color: char) -> Color {
    match color {
        'W' => Color::White,
        'U' => Color::Blue,
        'B' => Color::Black,
        'R' => Color::Red,
        'G' => Color::Green,
        'C' => Color::Colorless,
        _ => panic!("{color} is not a valid color!")
    }
}

fn parse_mana_cost(mut cost: &str) -> ManaCost {
    let mut mana: Vec<ManaSymbol> = vec![];

    //Since parse_mana_symbol modifies the string, 
    //this makes the iterator anew each time.
    //It won't be super expensive because char_indices doesn't allocate,
    // and even if it was, it wouldn't really matter because this is a build step.
    let mut chs = cost.char_indices();
    while let Some((i, ch)) = chs.next() {
        if ch == '{' {
            cost = &cost[(i + 1)..];
            let mana_symbol_src = scan_mana_symbol(&mut cost);
            mana.push(parse_mana_symbol(mana_symbol_src));
            chs = cost.char_indices();
        }
    }

    ManaCost(mana)
    
}

//NOTE: In general, this module panics instead of sensibly handling errors
fn main() -> io::Result<()> {
    let cards = std::env::args();
    //let cards = read_to_string("../temp/data/cards.json").expect("Bad data").to_string();
    //let json_cards: serde_json::Value = serde_json::from_str(&cards).expect("Not well formatted");
    // let card = json_cards[0].clone();
    // parse_card(card);
    dbg!(parse_mana_cost("{W}{W}{B}{3}"));
    
    Ok(())
}