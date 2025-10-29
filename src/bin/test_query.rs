use core::panic;
use minimal_storage::multitype_paged_storage::MultitypePagedStorage;
use project::data_model::card::{
    self, Card, CardRef, Color, ColorCombination, ManaCost, ManaSymbol, Supertype,
};
use project::dbs::allcards::AllCardsDb;
use project::dbs::allcards::cardref_key::card_ref_to_index;
use project::dbs::indexes::color_combination::ColorCombinationMaybe;
use serde_json;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::str::FromStr;
use std::time::Instant;
use std::u128;
use tree::sparse::structure::StoredTree;

const TESTS: usize = 1000;

fn main() -> io::Result<()> {
    let db_file = std::env::args()
        .nth(1)
        .expect("Usage: test_query <db_file>");

    let db = AllCardsDb::open(db_file).expect("Could not open <db_file>");

    let mut total_found = 0usize;

    let start = Instant::now();

    for i in 0..TESTS {
        total_found += db.query_color(&make_color_combination_maybe(i)).count();
    }

    let end = Instant::now();

    let test_dur_ms = (end - start).as_secs_f64() * 1000.0;
    let avg_res_per_search = (total_found as f64) / (TESTS as f64);
    let avg_time_per_search = test_dur_ms / (TESTS as f64);

    println!("Ran {TESTS} iterations in {test_dur_ms}ms, found {total_found} cumulative results
            (average of {avg_res_per_search} results and {avg_time_per_search}ms per search)");

    Ok(())
}

fn make_color_combination_maybe(index: usize) -> ColorCombinationMaybe {
    let mut value = index % 3usize.pow(6);

    let white = i_to_maybe_bool(value);
    value /= 3;
    let blue = i_to_maybe_bool(value);
    value /= 3;
    let black = i_to_maybe_bool(value);
    value /= 3;
    let red = i_to_maybe_bool(value);
    value /= 3;
    let green = i_to_maybe_bool(value);
    value /= 3;
    let colorless = i_to_maybe_bool(value);

    ColorCombinationMaybe {
        white,
        blue,
        black,
        red,
        green,
        colorless,
    }
}

fn i_to_maybe_bool(value: usize) -> Option<bool> {
    match value % 3 {
        0 => Some(false),
        1 => Some(true),
        2 => None,
        _ => unreachable!(),
    }
}
