use core::panic;
use minimal_storage::multitype_paged_storage::MultitypePagedStorage;
use project::data_model::card::{
    self, Card, CardRef, Color, ColorCombination, ManaCost, ManaSymbol, Supertype,
};
use project::dbs::allcards::AllCardsDb;
use project::dbs::allcards::cardref_key::card_ref_to_index;
use serde_json;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::str::FromStr;
use std::u128;
use tree::sparse::structure::StoredTree;

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
    return Card {
        name: card["name"]
            .as_str()
            .expect("Card name should be a string")
            .to_string(),
        mana_cost: parse_mana_cost(
            card["mana_cost"]
                .as_str()
                .expect("Mana cost should be a string"),
        ),
        mana_value_times_4: card["mana_value"]
            .as_str()
            .and_then(|x| x.parse::<f64>().ok())
            .filter(|x| (x * 4.0).fract() == 0.0)
            .map(|x| (x * 4.0) as usize)
            .expect("MVs should be a float encoded as a string, and no less than .25"),
        color: parse_color_combination(card["color"].as_str().expect("Bad Color Combo")),
        color_id: parse_color_combination(card["color_id"].as_str().expect("Bad Color Combo")),
        super_types: card["super_types"]
            .as_str()
            .expect("super_types should be a string")
            .split(", ")
            .filter(|x| !x.is_empty())
            .map(|x| match x {
                "Basic" => Supertype::Basic,
                "Legendary" => Supertype::Legendary,
                "Elite" => Supertype::Elite,
                "Ongoing" => Supertype::Ongoing,
                "Host" => Supertype::Host,
                "World" => Supertype::World,
                "Snow" => Supertype::Snow,
                _ => panic!("Unknown supertype '{x}'"),
            })
            .collect(),
        rarity: match card["rarity"].as_str().expect("Bad Rarity") {
            "common" => card::Rarity::Common,
            "uncommon" => card::Rarity::Uncommon,
            "rare" => card::Rarity::Rare,
            "mythic" => card::Rarity::Mythic,
            "special" => card::Rarity::Special,
            other => panic!("Unexpected rarity value {other}"),
        },
        oracle_text: card["oracle_text"]
            .as_str()
            .expect("Oracle text should be a string")
            .to_string(),
        power: stringified_num(&card["power"]),
        toughness: stringified_num(&card["toughness"]),
        types: card["types"]
            .as_str()
            .expect("Bad types")
            .split(", ")
            .filter(|x| !x.is_empty())
            .map(String::from)
            .collect(),
        subtypes: card["subtypes"]
            .as_str()
            .expect("Bad types")
            .split(", ")
            .filter(|x| !x.is_empty())
            .map(String::from)
            .collect(),
        loyalty: stringified_num(&card["loyalty"]),
        defense: stringified_num(&card["defense"]),
        sets_released: card["sets_released"]
            .as_str()
            .expect("Bad types")
            .split(", ")
            .map(String::from)
            .collect(),
        game_changer: card["game_changer"].as_str().expect("Bad game_changer") == "true",
    };
    // for (key, value) in card_obj {

    // }
}

fn stringified_num<T: FromStr + Default>(card: &serde_json::Value) -> T {
    let str = card
        .as_str()
        .expect("Stringified numbers should be stored as a string!");
    if str.is_empty() {
        return T::default();
    }

    match str.parse() {
        Ok(t) => t,
        Err(e) => panic!(
            "{str} cannot be interpreted as a {}.",
            std::any::type_name::<T>()
        ),
    }
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

    match src {
        "S" => return ManaSymbol::Snow,
        "X" => return ManaSymbol::Variable(card::ManaVariable::X),
        "Y" => return ManaSymbol::Variable(card::ManaVariable::Y),
        "Z" => return ManaSymbol::Variable(card::ManaVariable::Z),
        "D" => return ManaSymbol::LandDrop,
        "L" => return ManaSymbol::Legendary,
        "HW" => return ManaSymbol::HalfWhite,
        "1000000" => return ManaSymbol::OneMillionGenericMana,
        _ => {}
    }

    if src.chars().all(|x| x.is_ascii_digit()) {
        return ManaSymbol::GenericNumber(src.parse().unwrap());
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
        return ManaSymbol::ConventionalColored {
            phyrexian,
            split_two_generic,
            color,
            split_color,
        };
    } else if colors.len() == 1 {
        let split_color = None;
        let color = colors.pop().unwrap();
        return ManaSymbol::ConventionalColored {
            phyrexian,
            split_two_generic,
            color,
            split_color,
        };
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
        _ => panic!("{color} is not a valid color!"),
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

fn process_sets_map(json: serde_json::Value) -> BTreeMap<String, CardRef> {
    let mut map = BTreeMap::new();

    for set_spec in json.as_array().expect("Sets JSON should be an array") {
        let set_spec = set_spec.as_object().expect("Each set should be an object");

        for (setcode, v) in set_spec.iter() {
            for collectors_num in v.as_array().expect("Sets should be an arr") {
                let v = collectors_num
                    .as_array()
                    .expect("Collector numbers should be an array");
                assert!(
                    v.len() == 2,
                    "Collector numbers should be an array of 2 items"
                );

                let name = v
                    .get(0)
                    .unwrap()
                    .as_str()
                    .expect("Names should be a string");
                let collector_number = v
                    .get(1)
                    .unwrap()
                    .as_str()
                    .expect("Collector number should be a string");

                map.insert(
                    name.to_string(),
                    CardRef {
                        set: setcode.to_string(),
                        collector_number: collector_number.parse().unwrap(),
                        printing: None,
                    },
                );
            }
        }
    }

    map
}

//NOTE: In general, this module panics instead of sensibly handling errors
fn main() -> io::Result<()> {
    let cards_file = std::env::args()
        .nth(1)
        .expect("Usage: build_card_db <cards_file> <sets_file> <db_file>");
    let sets_file = std::env::args()
        .nth(2)
        .expect("Usage: build_card_db <cards_file> <sets_file> <db_file>");
    let db_file = std::env::args()
        .nth(3)
        .expect("Usage: build_card_db <cards_file> <sets_file> <db_file>");

    let rdr = BufReader::new(File::open(cards_file).expect("Can't open <cards_file>"));
    let json_cards: serde_json::Value =
        serde_json::from_reader(rdr).expect("Bad data in <cards_file>");

    //try to remove the old database. no sweat if it doesn't work.
    let _ = std::fs::remove_file(&db_file);

    let db = AllCardsDb::open(db_file).expect("Could not open <db_file>");

    let cards_arr = match json_cards {
        serde_json::Value::Array(values) => values,
        _ => panic!("Cards file should be a JSON array"),
    };

    let rdr = BufReader::new(File::open(sets_file).expect("Can't open <cards_file>"));
    let sets = serde_json::from_reader::<_, serde_json::Value>(rdr)
        .map(process_sets_map)
        .expect("Bad data in <sets_file>");

    let card_last_idx = cards_arr.len() - 1;

    let mut cards_already_seen = HashSet::new();

    for (i, card) in cards_arr.into_iter().enumerate() {
        let card = parse_card(card);
        if cards_already_seen.contains(&card) {
            continue;
        } else {
            cards_already_seen.insert(card.clone());
        }
        let cardref = sets
            .get(&card.name)
            .expect(&format!("'{}' must have a collector's number", card.name));
        db.add(cardref, card);

        eprint!("{i}/{card_last_idx} \u{1b}[0E");
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::{
        fs::File,
        io::{BufReader, Read},
    };

    use minimal_storage::serialize_min::{DeserializeFromMinimal, SerializeMinimal};
    use project::data_model::card::Card;

    #[test]
    fn serde_deserde_cards() {
        let cards_file = "data/cards.json";

        let rdr = BufReader::new(File::open(cards_file).expect("Can't open <cards_file>"));
        let json_cards: Vec<serde_json::Value> =
            serde_json::from_reader(rdr).expect("Bad data in <cards_file>");

        let mut store_buf = Vec::<u8>::new();

        for (i, card) in json_cards.into_iter().enumerate().skip(1) {
            eprintln!("Card {i}...");
            let card = crate::parse_card(card);
            card.minimally_serialize(&mut store_buf, ()).unwrap();
            eprintln!(
                "{}",
                store_buf
                    .iter()
                    .map(|x| format!("{x:0<2x} "))
                    .collect::<String>()
            );
            let card_roundtrip = Card::deserialize_minimal(&mut &store_buf[..], ()).unwrap();

            debug_assert_eq!(card, card_roundtrip);
            store_buf.truncate(0);
        }
    }
}
