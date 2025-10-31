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

fn main() -> io::Result<()> {
    let db_file = std::env::args()
        .nth(1)
        .expect("Usage: test_query <db_file>");

    let db = AllCardsDb::open(db_file).expect("Could not open <db_file>");

    let mut total_found = 0usize;

    let start = Instant::now();

    for i in 0..TESTS {
        total_found += db.query_mana(&make_mana_query(i)).count();
    }

    let end = Instant::now();

    let test_dur_ms = (end - start).as_secs_f64() * 1000.0;
    let avg_res_per_search = (total_found as f64) / (TESTS as f64);
    let avg_time_per_search = test_dur_ms / (TESTS as f64);

    println!(
        "Ran {TESTS} iterations in {test_dur_ms}ms, found {total_found} cumulative results
            (average of {avg_res_per_search} results and {avg_time_per_search}ms per search)"
    );

    Ok(())
}

fn make_mana_query(mut index: usize) -> ManaCostCount::Query {    
    let field_querying = index % 12;
    index /= 12;
    let value_querying = index % 5;

    let mut num_white = 0..=usize::MAX;
    let mut num_blue = 0..=usize::MAX;
    let mut num_black = 0..=usize::MAX;
    let mut num_red = 0..=usize::MAX;
    let mut num_green = 0..=usize::MAX;
    let mut num_colorless = 0..=usize::MAX;
    let mut num_generic = 0..=usize::MAX;
    let mut num_any_phyrexian = 0..=usize::MAX;
    let mut num_any_color_split = 0..=usize::MAX;
    let mut num_any_split_generic = 0..=usize::MAX;
    let mut num_variables_used = 0..=usize::MAX;
    let mut num_odd_edge_case_symbols = 0..=usize::MAX;

    match field_querying {
        0 => num_white = value_querying..=value_querying,
        1 => num_blue = value_querying..=value_querying,
        2 => num_black = value_querying..=value_querying,
        3 => num_red = value_querying..=value_querying,
        4 => num_green = value_querying..=value_querying,
        5 => num_colorless = value_querying..=value_querying,
        6 => num_generic = value_querying..=value_querying,
        7 => num_any_phyrexian = value_querying..=value_querying,
        8 => num_any_color_split = value_querying..=value_querying,
        9 => num_any_split_generic = value_querying..=value_querying,
        10 => num_variables_used = value_querying..=value_querying,
        11 => num_odd_edge_case_symbols = value_querying..=value_querying,
        _ => unreachable!()
    }

    ManaCostCount::Query {
        num_white,
        num_blue,
        num_black,
        num_red,
        num_green,
        num_colorless,
        num_generic,
        num_any_phyrexian,
        num_any_split_generic,
        num_any_color_split,
        num_variables_used,
        num_odd_edge_case_symbols,
    }
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
