use core::panic;
use minimal_storage::multitype_paged_storage::MultitypePagedStorage;
use project::data_model::card::{
    self, Card, CardRef, Color, ColorCombination, ManaCost, ManaSymbol, Supertype,
};
use project::dbs::allcards::AllCardsDb;
use project::dbs::allcards::cardref_key::card_ref_to_index;
use project::dbs::indexes::color_combination::ColorCombinationMaybe;
use project::dbs::indexes::mana_cost::ManaCostCount;
use serde_json;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::str::FromStr;
use std::time::Instant;
use std::u128;
use tree::sparse::structure::StoredTree;

const TESTS: usize = 10_000;
const PAGE_COUNT: usize = 50;

fn main() -> io::Result<()> {
    let db_file = std::env::args()
        .nth(1)
        .expect("Usage: test_query <db_file> <search>");

    let search_term = std::env::args()
        .nth(2)
        .expect("Usage: test_query <db_file> <search>");

    let db = AllCardsDb::open(db_file).expect("Could not open <db_file>");

    let start = Instant::now();

    let mut cards = db.fulltext_search(&search_term, 0.1);

    let end = Instant::now();

    let mut cards_taken = 0;
    while let Some(card) = cards.pop() {
        cards_taken += 1;
        if cards_taken >= PAGE_COUNT {
            break;
        }

        let card_ref = db.get_card(*card).unwrap();
        let card = card_ref.as_ref();

        let name = &card.name;
        let oracle = &card.oracle_text;

        println!("{name}: {oracle}\n");
    }

    println!(
        "{} results in {}s", cards.len(), (end - start).as_secs_f64()
    );

    Ok(())
}
