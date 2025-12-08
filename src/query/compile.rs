use std::{
    collections::{BTreeMap, HashMap},
    mem::{Discriminant, discriminant},
    ops::Not,
};

use tree::tree_traits::{MultidimensionalKey, MultidimensionalParent};

use crate::{
    color_combo,
    data_model::card::{Card, ColorCombination},
    dbs::{
        allcards::AllCardsDb,
        indexes::{
            color_combination::ColorCombinationMaybe,
            mana_cost::{self, ManaCostCount},
            stats::card_stats,
            string_lpm::LongestPrefixMatch,
            string_trigram::trigram,
        },
    },
    query::{
        err_warn_support::{Message, MessageSink},
        parse::{SearchQuery, SearchQueryTree, SearchTerm, parse_str},
    },
};

#[derive(Debug)]
pub enum DbQueryIndex {
    Color(ColorCombinationMaybe),
    ColorId(ColorCombinationMaybe),
    CardStats(card_stats::Query),
    Type(LongestPrefixMatch),
    ManaCost(mana_cost::ManaCostCount::Query),
    NameExact(LongestPrefixMatch),
    Empty,
}
impl DbQueryIndex {
    fn intersected_color_id(self, c1: &ColorCombinationMaybe) -> Option<Self> {
        match self {
            Self::ColorId(c2) => Some(Self::ColorId(c1.intersect(&c2)?)),
            _ => panic!("intersected_color_id called on not color id!"),
        }
    }
    fn intersected_color(self, c1: &ColorCombinationMaybe) -> Option<Self> {
        match self {
            Self::Color(c2) => Some(Self::Color(c1.intersect(&c2)?)),
            _ => panic!("intersected_color called on not color!"),
        }
    }
    fn intersected_card_stats(self, c1: &card_stats::Query) -> Option<Self> {
        match self {
            Self::CardStats(c2) => Some(Self::CardStats(c1.intersect(&c2)?)),
            _ => panic!("intersected_card_stats called on not card stats!"),
        }
    }
    fn intersected_mana_cost(self, c1: &ManaCostCount::Query) -> Option<Self> {
        match self {
            Self::ManaCost(c2) => Some(Self::ManaCost(c1.intersect(&c2)?)),
            _ => panic!("intersected_mana_cost called on not mana cost!"),
        }
    }
}

#[derive(Debug)]
pub enum DbQueryFieldParam<'s> {
    Color(ColorCombinationMaybe),
    ColorId(ColorCombinationMaybe),
    Type(&'s str),
    TypeNot(&'s str),
    CardStats(card_stats::Query),
    ManaCost(mana_cost::ManaCostCount::Query),
    NameIncludes(&'s str),
    NameExact(&'s str),
    NameNotIncludes(&'s str),
    NotNameExact(&'s str),
    OracleTextIncludes(&'s str),
    OracleTextNotIncludes(&'s str),
}

impl<'s> DbQueryFieldParam<'s> {
    pub fn into_index_param(self) -> Option<DbQueryIndex> {
        match self {
            DbQueryFieldParam::Color(c) => Some(DbQueryIndex::Color(c)),
            DbQueryFieldParam::ColorId(c) => Some(DbQueryIndex::ColorId(c)),
            DbQueryFieldParam::CardStats(s) => Some(DbQueryIndex::CardStats(s)),
            DbQueryFieldParam::ManaCost(m) => Some(DbQueryIndex::ManaCost(m)),
            DbQueryFieldParam::NameExact(n) => Some(DbQueryIndex::NameExact(
                LongestPrefixMatch::new_prefix(n.to_ascii_lowercase()),
            )),
            DbQueryFieldParam::Type(n) => Some(DbQueryIndex::Type(LongestPrefixMatch::new_prefix(
                n.to_ascii_lowercase(),
            ))),
            _ => None,
        }
    }

    fn matches_card(&self, card: &Card) -> bool {
        match self {
            DbQueryFieldParam::Color(color) => card.color.is_contained_in(color),
            DbQueryFieldParam::ColorId(color_id) => card.color_id.is_contained_in(color_id),
            DbQueryFieldParam::Type(t) => card
                .types
                .iter()
                .chain(card.subtypes.iter())
                .any(|c_t| scryfall_ish_string_includes(&c_t, *t)),
            DbQueryFieldParam::TypeNot(t) => card.types.iter().any(|c_t| c_t.contains(t)).not(),
            DbQueryFieldParam::CardStats(query) => {
                (card.game_changer as u8).is_contained_in(&query.game_changer)
                    && card.defense.is_contained_in(&query.defense)
                    && card.loyalty.as_repr_usize().is_contained_in(&query.loyalty)
                    && card
                        .toughness
                        .as_repr_usize()
                        .is_contained_in(&query.toughness)
                    && card.power.as_repr_usize().is_contained_in(&query.power)
                    && card
                        .mana_value_times_4
                        .is_contained_in(&query.mana_value_quarters)
            }
            DbQueryFieldParam::ManaCost(query) => {
                ManaCostCount::Key::new(&card.mana_cost).is_contained_in(&query)
            }
            DbQueryFieldParam::NameIncludes(t) => scryfall_ish_string_includes(&card.name, *t),
            DbQueryFieldParam::NameExact(n) => card.name == *n,
            DbQueryFieldParam::NameNotIncludes(t) => !card.name.contains(*t),
            DbQueryFieldParam::NotNameExact(n) => card.name != *n,
            DbQueryFieldParam::OracleTextIncludes(t) => {
                scryfall_ish_string_includes(&card.oracle_text, *t)
            }
            DbQueryFieldParam::OracleTextNotIncludes(t) => !card.oracle_text.contains(*t),
        }
    }
}

fn scryfall_ish_string_includes(haystack: &str, needle: &str) -> bool {
    for i in 0..(haystack.len().saturating_sub(needle.len()) + 1) {
        let needed_chars = needle
            .as_bytes()
            .iter()
            .map(|x| x.to_ascii_lowercase())
            .filter(|x| !x.is_ascii_whitespace());
        let present_chars = haystack.as_bytes()[i..]
            .iter()
            .map(|x| x.to_ascii_lowercase())
            .filter(|x| !x.is_ascii_whitespace());

        if needed_chars.zip(present_chars).all(|(a, b)| a == b) {
            return true;
        }
    }
    return false;
}

#[derive(Debug)]
pub struct DbQuery<'s> {
    index: Option<DbQueryIndex>,
    tree: DbQueryTree<'s>,
}

impl DbQuery<'_> {
    pub fn query_db<'a>(&'a self, db: &'a AllCardsDb) -> Box<dyn Iterator<Item = Card> + 'a> {
        match &self.index {
            Some(DbQueryIndex::CardStats(c)) => {
                return Box::new(db.query_stats(c).filter(|x| self.tree.matches_card(x)));
            }
            Some(DbQueryIndex::Color(c)) => {
                return Box::new(db.query_color(c).filter(|x| self.tree.matches_card(x)));
            }
            Some(DbQueryIndex::ColorId(c)) => {
                return Box::new(db.query_color_id(c).filter(|x| self.tree.matches_card(x)));
            }
            Some(DbQueryIndex::ManaCost(c)) => {
                return Box::new(db.query_mana(c).filter(|x| self.tree.matches_card(x)));
            }
            Some(DbQueryIndex::NameExact(c)) => {
                return Box::new(db.query_name(c).filter(|x| self.tree.matches_card(x)));
            }
            Some(DbQueryIndex::Type(t)) => {
                return Box::new(db.query_type(t).filter(|x| self.tree.matches_card(x)));
            }
            Some(DbQueryIndex::Empty) => return Box::new(std::iter::empty()),
            None => {
                return Box::new(db.all_cards().filter(|x| self.tree.matches_card(x)));
            }
        };
    }
}

#[derive(Debug)]
pub enum DbQueryTree<'s> {
    And(Vec<DbQueryTree<'s>>),
    Or(Vec<DbQueryTree<'s>>),
    Term(DbQueryFieldParam<'s>),
}

impl DbQueryTree<'_> {
    pub fn matches_card(&self, card: &Card) -> bool {
        match self {
            DbQueryTree::And(ands) => ands.iter().all(|x| x.matches_card(card)),
            DbQueryTree::Or(ors) => ors.iter().any(|x| x.matches_card(card)),
            DbQueryTree::Term(field) => field.matches_card(card),
        }
    }
}

pub fn build_search_query<'q>(
    query: &'q str,
    msgs: &'q impl MessageSink,
) -> Result<DbQuery<'q>, String> {
    let Some(sq) = parse_str(query, msgs) else {
        return Err(query.to_string());
    };

    compile(&sq, msgs).unwrap_or_else(|| Err(query.to_string()))
}

fn compile<'q, 'c>(
    q: &'c SearchQuery<'q>,
    msgs: &'c impl MessageSink,
) -> Option<Result<DbQuery<'q>, String>> {
    'check_is_simple: loop {
        match &q.query {
            SearchQueryTree::And(items) => {
                let mut q = String::new();
                for f in items.iter() {
                    match f.query {
                        SearchQueryTree::Term(SearchTerm::Term(t)) => {
                            if !q.is_empty() {
                                q.push(' ');
                            }
                            q.push_str(t);
                        }
                        _ => break 'check_is_simple,
                    }
                }
                return Some(Err(q));
            }
            SearchQueryTree::Term(SearchTerm::Term(t)) => return Some(Err(t.to_string())),
            _ => break 'check_is_simple,
        }
    }

    let index_field = match &q.query {
        SearchQueryTree::And(_) => find_index_field(&q.query, q.source_range.start, msgs),
        SearchQueryTree::Term(_) => find_index_field(&q.query, q.source_range.start, msgs),
        SearchQueryTree::Or(_) => None, //we can't have an exclusive index when the root is an OR
    };

    Some(Ok(DbQuery {
        index: index_field,
        tree: tree_to_tree(&q.query, q.source_range.start, msgs)?,
    }))
}

fn tree_to_tree<'q, 'c>(
    t: &'c SearchQueryTree<'q>,
    byte_index: usize,
    compile_errs: &'c impl MessageSink,
) -> Option<DbQueryTree<'q>> {
    Some(match t {
        //TODO: optimize AND by combining multiple queries on the same field into single narrowed queries
        SearchQueryTree::And(items) => DbQueryTree::And(
            items
                .iter()
                .filter_map(|q| tree_to_tree(&q.query, q.source_range.start, compile_errs))
                .collect::<Vec<_>>(),
        ),
        SearchQueryTree::Or(items) => DbQueryTree::Or(
            items
                .iter()
                .filter_map(|q| tree_to_tree(&q.query, q.source_range.start, compile_errs))
                .collect::<Vec<_>>(),
        ),
        SearchQueryTree::Term(term) => {
            DbQueryTree::Term(term_to_field(term, byte_index, compile_errs)?)
        }
    })
}

fn find_index_field<'q, 'c>(
    q: &'c SearchQueryTree<'q>,
    byte_index: usize,
    compile_errs: &'c impl MessageSink,
) -> Option<DbQueryIndex> {
    match q {
        //due to the way that the DB indices work, we can't really OR query with an index
        SearchQueryTree::Or(_) => None,
        SearchQueryTree::And(items) => {
            //no nested AND lists because we'll've already flattened.
            let mut most_common_terms =
                HashMap::<Discriminant<_>, (Vec<DbQueryIndex>, usize)>::new();

            for field in items.iter() {
                let Some(field_index) =
                    find_index_field(&field.query, field.source_range.start, compile_errs)
                else {
                    continue;
                };

                let this_mct = most_common_terms
                    .entry(discriminant(&field_index))
                    .or_default();

                this_mct.1 = field.source_range.start;
                this_mct.0.push(field_index);
            }

            let mut most_common_term = (0usize, None, 0);
            for (term_collection, err_i) in most_common_terms.into_values() {
                if term_collection.len() > most_common_term.0 {
                    most_common_term = (term_collection.len(), Some(term_collection), err_i);
                }
            }

            let terms = most_common_term.1?;
            let terms_intersect = match intersect_index_terms(
                terms,
                compile_errs,
                most_common_term.2,
            ) {
                Some(i) => i,
                None => {
                    compile_errs.send(Message {
                        msg_type: super::err_warn_support::MessageSeverity::Warning,
                        msg_content: format!("This combination of queries will never have any search results. Try relaxing some of your filters."),
                        byte_pos: byte_index,
                        source_phase_index: 2
                    });
                    DbQueryIndex::Empty
                }
            };
            Some(terms_intersect)
        }
        SearchQueryTree::Term(search_term) => {
            term_to_field(search_term, byte_index, compile_errs).and_then(|x| x.into_index_param())
        }
    }
}

///
/// Panics on empty vec. Does not function when enum variants are mixed
fn intersect_index_terms<'q, 'c>(
    mut terms: Vec<DbQueryIndex>,
    msgs: &'c impl MessageSink,
    last_index_errors: usize,
) -> Option<DbQueryIndex> {
    if terms.len() <= 1 {
        return Some(terms.pop().unwrap());
    }

    let mut current_combo = match terms.pop().unwrap() {
        DbQueryIndex::Empty => return Some(DbQueryIndex::Empty),
        //can't query for more than one type at a time with the index, so simply filter it :)
        i @ DbQueryIndex::Type(_) => return Some(i),
        i @ DbQueryIndex::ColorId(_)
        | i @ DbQueryIndex::Color(_)
        | i @ DbQueryIndex::CardStats(_)
        | i @ DbQueryIndex::ManaCost(_) => i,
        DbQueryIndex::NameExact(t) => {
            //if there's more than one NameExact (i.e. the terms vector
            // is nonempty after taking one out),
            // then we warn the user that they're filtering to nothing
            if terms.is_empty() {
                return Some(DbQueryIndex::NameExact(t));
            } else {
                msgs.send(Message {
                    msg_type: super::err_warn_support::MessageSeverity::Warning,
                    msg_content: format!("You're filtering on exact names (!\"...\") multiple times, which will result in no results"),
                    byte_pos: last_index_errors,
                    source_phase_index: 2
                });
                return Some(DbQueryIndex::Empty);
            }
        }
    };

    for other_field in terms {
        match other_field {
            DbQueryIndex::ColorId(c) => {
                current_combo = current_combo.intersected_color_id(&c)?;
            }
            DbQueryIndex::Color(c) => {
                current_combo = current_combo.intersected_color(&c)?;
            }
            DbQueryIndex::CardStats(query) => {
                current_combo = current_combo.intersected_card_stats(&query)?;
            }
            DbQueryIndex::ManaCost(query) => {
                current_combo = current_combo.intersected_mana_cost(&query)?;
            }
            _ => unreachable!(),
        }
    }

    return Some(current_combo);
}

pub fn term_to_field<'q, 'c>(
    term: &'c SearchTerm<'q>,
    byte_index: usize,
    compile_errs: &'c impl MessageSink,
) -> Option<DbQueryFieldParam<'q>> {
    match term {
        SearchTerm::Term(s) => Some(DbQueryFieldParam::NameIncludes(s)),
        SearchTerm::NegTerm(s) => Some(DbQueryFieldParam::NameNotIncludes(s)),
        SearchTerm::Exact(s) => Some(DbQueryFieldParam::NameExact(s)),
        SearchTerm::NegExact(s) => Some(DbQueryFieldParam::NotNameExact(s)),
        SearchTerm::KeyVal(k, v) => key_op_to_field(k, BinCmp::Eq, v, byte_index, compile_errs),
        SearchTerm::KeyNeq(k, v) => key_op_to_field(k, BinCmp::Neq, v, byte_index, compile_errs),
        SearchTerm::KeyGt(k, v) => key_op_to_field(k, BinCmp::Gt, v, byte_index, compile_errs),
        SearchTerm::KeyLt(k, v) => key_op_to_field(k, BinCmp::Lt, v, byte_index, compile_errs),
        SearchTerm::KeyLte(k, v) => key_op_to_field(k, BinCmp::Lte, v, byte_index, compile_errs),
        SearchTerm::KeyGte(k, v) => key_op_to_field(k, BinCmp::Gte, v, byte_index, compile_errs),
    }
}

#[derive(PartialEq, Debug)]
enum BinCmp {
    Neq,
    Eq,
    Gte,
    Gt,
    Lt,
    Lte,
}

fn key_op_to_field<'q>(
    k: &'q str,
    op: BinCmp,
    v: &'q str,
    byte_index: usize,
    compile_errs: &impl MessageSink,
) -> Option<DbQueryFieldParam<'q>> {
    if v == "" {
        return None;
    }
    match k {
        "o" | "oracle" => match op {
            BinCmp::Neq => Some(DbQueryFieldParam::OracleTextNotIncludes(v)),
            _ => warn_interp_cmp_as_eq(&compile_errs, k, op, v, byte_index)
                .map(DbQueryFieldParam::OracleTextIncludes),
        },
        "t" | "type" => match op {
            BinCmp::Neq => Some(DbQueryFieldParam::TypeNot(v)),
            _ => warn_interp_cmp_as_eq(&compile_errs, k, op, v, byte_index)
                .map(DbQueryFieldParam::Type),
        },
        "mv" | "manavalue" => {
            if v == "even" || v == "odd" {
                compile_errs.send(Message {
                    msg_type: super::err_warn_support::MessageSeverity::Error,
                    msg_content: format!("We don't yet support mv:even or mv:odd. Sorry!"),
                    byte_pos: byte_index,
                    source_phase_index: 2,
                });
                return None;
            }
            let Ok(number_value) = v.parse::<usize>() else {
                compile_errs.send(Message {
                    msg_type: super::err_warn_support::MessageSeverity::Error,
                    msg_content: format!("{v:?} isn't a valid integer"),
                    byte_pos: byte_index,
                    source_phase_index: 2,
                });
                return None;
            };

            let mana_value_quarters = number_value * 4;

            let mv_range = match op {
                BinCmp::Neq | BinCmp::Gt => (mana_value_quarters + 1)..=usize::MAX,
                BinCmp::Eq => mana_value_quarters..=mana_value_quarters,
                BinCmp::Gte => mana_value_quarters..=usize::MAX,
                BinCmp::Lt => match mana_value_quarters.checked_sub(1).map(|x| 0..=x) {
                    Some(t) => t,
                    None => {
                        compile_errs.send(Message {
                            msg_type: super::err_warn_support::MessageSeverity::Error,
                            msg_content: format!("There are no cards with negative mana value"),
                            byte_pos: byte_index,
                            source_phase_index: 2,
                        });
                        return None;
                    }
                },
                BinCmp::Lte => 0..=mana_value_quarters,
            };
            Some(DbQueryFieldParam::CardStats(card_stats::Query {
                mana_value_quarters: mv_range,
                ..card_stats::Query::UNIVERSE
            }))
        }

        "c" | "color" | "id" | "identity" => {
            let value = color_name(v, byte_index, compile_errs)?;

            //scryfall's interpretation of <, <=, >, and >= isn't defined in their docs.
            // this is pretty close to what they do i think?
            let value_comparison = match op {
                BinCmp::Lt | BinCmp::Neq => ColorCombinationMaybe {
                    white: value.white.then_some(false),
                    blue: value.blue.then_some(false),
                    black: value.black.then_some(false),
                    red: value.red.then_some(false),
                    green: value.green.then_some(false),
                    colorless: value.colorless.then_some(false),
                },
                BinCmp::Eq | BinCmp::Lte | BinCmp::Gte | BinCmp::Gt => ColorCombinationMaybe {
                    white: value.white.then_some(true),
                    blue: value.blue.then_some(true),
                    black: value.black.then_some(true),
                    red: value.red.then_some(true),
                    green: value.green.then_some(true),
                    colorless: value.colorless.then_some(true),
                },
            };

            if k.starts_with('c') {
                return Some(DbQueryFieldParam::Color(value_comparison));
            } else {
                return Some(DbQueryFieldParam::ColorId(value_comparison));
            }
        }
        some_other_key => {
            compile_errs.send(Message {
                msg_type: super::err_warn_support::MessageSeverity::Error,
                msg_content: format!("We don't handle the {some_other_key:?} keyword yet, sorry! We're working on complete Scryfall coverage."),
                byte_pos: byte_index,
                source_phase_index: 2
            });
            None
        }
    }
}

fn warn_interp_cmp_as_eq<'q>(
    compile_errs: &impl MessageSink,
    k: &str,
    op: BinCmp,
    v: &'q str,
    byte_pos: usize,
) -> Option<&'q str> {
    if op != BinCmp::Eq {
        compile_errs.send(Message {
            msg_type: super::err_warn_support::MessageSeverity::Warning,
            msg_content: format!("You can't use comparison operators on the {k:?} keyword; it's being interpreted as '{k}:' instead"),
            byte_pos,
            source_phase_index: 2
        });
    }

    Some(v)
}

fn color_name(
    color: &str,
    byte_index: usize,
    compile_errs: &impl MessageSink,
) -> Option<ColorCombination> {
    Some(match color {
        "white" => color_combo!(w),
        "blue" => color_combo!(u),
        "black" => color_combo!(b),
        "red" => color_combo!(r),
        "green" => color_combo!(g),
        "colorless" => color_combo!(c),
        "azorius" => color_combo!(w  u ),
        "dimir" => color_combo!(u  b ),
        "rakdos" => color_combo!(b  r ),
        "gruul" => color_combo!(r  g ),
        "selesnya" => color_combo!(g  w ),
        "ojutai" => color_combo!(w  u ),
        "silumgar" => color_combo!(u  b ),
        "kolaghan" => color_combo!(b  r ),
        "atarka" => color_combo!(r  g ),
        "dromoka" => color_combo!(g  w ),
        "orzhov" => color_combo!(w  b ),
        "izzet" => color_combo!(u  r ),
        "golgari" => color_combo!(b  g ),
        "boros" => color_combo!(r  w ),
        "simic" => color_combo!(g  u ),
        "lorehold" => color_combo!(r  w ),
        "prismari" => color_combo!(u  r ),
        "quandrix" => color_combo!(g  u ),
        "silverquill" => color_combo!(w  b ),
        "witherbloom" => color_combo!(b  g ),
        "bant" => color_combo!(g  w  u ),
        "esper" => color_combo!(w  u  b ),
        "grixis" => color_combo!(u  b  r ),
        "jund" => color_combo!(b  r  g ),
        "naya" => color_combo!(r  g  w ),
        "brokers" | "broker" => color_combo!(g  w  u ),
        "obscura" => color_combo!(w  u  b ),
        "maestros" | "maestro" => color_combo!(u  b  r ),
        "riveteers" | "riveteer" => color_combo!(b  r  g ),
        "cabaretti" => color_combo!(r  g  w ),
        "abzan" => color_combo!(w  b  g ),
        "jeskai" => color_combo!(u  r  w ),
        "sultai" => color_combo!(b  g  u ),
        "mardu" => color_combo!(r  w  b ),
        "temur" => color_combo!(g  u  r ),
        "savai" => color_combo!(r  w  b ),
        "ketria" => color_combo!(g  u  r ),
        "indatha" => color_combo!(w  b  g ),
        "raugrin" => color_combo!(u  r  w ),
        "zagoth" => color_combo!(b  g  u ),
        "yore" | "artifice" => color_combo!(w  u  b  r ),
        "glint" | "chaos" => color_combo!(u  b  r  g ),
        "dune" | "aggression" => color_combo!(b  r  g  w ),
        "ink" | "altruism" => color_combo!(r  g  w  u ),
        "witch" | "growth" => color_combo!(g  w  u  b ),
        "multicolor" => {
            compile_errs.send(Message {
                byte_pos: byte_index,
                msg_type: crate::query::err_warn_support::MessageSeverity::Error,
                msg_content: format!(
                    "Sorry, filtering for multicolor cards isn't supported for now."
                ),
                source_phase_index: 2,
            });
            return None;
        }
        color_letters => {
            let mut f = ColorCombination::default();
            for (i, c) in color_letters.char_indices() {
                match c {
                    'w' => f.white = true,
                    'u' => f.blue = true,
                    'b' => f.black = true,
                    'r' => f.red = true,
                    'g' => f.green = true,
                    'c' => f.colorless = true,
                    'm' => {
                        compile_errs.send(Message {
                            byte_pos: byte_index + i,
                            msg_type: crate::query::err_warn_support::MessageSeverity::Error,
                            msg_content: format!(
                                "Sorry, filtering for multicolor cards isn't supported for now."
                            ),
                            source_phase_index: 2,
                        });
                        return None;
                    }
                    _ => {
                        compile_errs.send(Message {
                            byte_pos: byte_index,
                            msg_type: crate::query::err_warn_support::MessageSeverity::Error,
                            msg_content: format!("'{color_letters}' is not a valid color. See https://scryfall.com/docs/syntax#colors"),
                            source_phase_index: 2
                        });
                    }
                }
            }
            f
        }
    })
}

pub fn flatten(q: &mut SearchQueryTree) {
    match q {
        SearchQueryTree::And(items) => {
            for i in 0..(items.len()) {
                if items[i].query.is_and() {
                    let first_item = items[i]
                        .query
                        .children_mut()
                        .expect("after is_and(), should always have children")
                        .pop()
                        .expect("ANDs should never be empty -- impossible by the fact that it would be a Term if so.");

                    let cell = std::mem::replace(&mut items[i], first_item);

                    items.extend(cell.query.into_children().into_iter().flatten());
                }
            }
            for f in items.iter_mut() {
                flatten(&mut f.query);
            }
        }
        SearchQueryTree::Or(items) => {
            for i in 0..(items.len()) {
                if items[i].query.is_or() {
                    let first_item = items[i]
                        .query
                        .children_mut()
                        .expect("after is_or(), should always have children")
                        .pop()
                        .expect("ORs should never be empty -- syntactically impossible");

                    let cell = std::mem::replace(&mut items[i], first_item);

                    items.extend(cell.query.into_children().into_iter().flatten());
                }
            }
            for f in items.iter_mut() {
                flatten(&mut f.query);
            }
        }
        SearchQueryTree::Term(_) => {}
    }
}

#[cfg(test)]
mod test {
    use crate::query::compile::scryfall_ish_string_includes;

    #[test]
    pub fn test() {
        assert!(scryfall_ish_string_includes("haystack", "h"));
        assert!(scryfall_ish_string_includes("haystack", "haystack"));
        assert!(!scryfall_ish_string_includes("haystack", "x"));
    }
}
