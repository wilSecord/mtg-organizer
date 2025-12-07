use tree::tree_traits::MultidimensionalParent;

use crate::{
    color_combo,
    data_model::card::ColorCombination,
    dbs::indexes::{color_combination::ColorCombinationMaybe, mana_cost, stats::card_stats},
    query::{
        err_warn_support::{Message, MessageSink},
        parse::{SearchQuery, SearchQueryTree, SearchTerm, parse_str},
    },
};

pub enum DbQueryIndex<'s> {
    Color(ColorCombinationMaybe),
    ColorId(ColorCombinationMaybe),
    Type(&'s str),
    CardStats(card_stats::Query),
    ManaCost(mana_cost::ManaCostCount::Query),
    NameExact(&'s str),
}

pub enum DbQueryFieldParam<'s> {
    Color(ColorCombinationMaybe),
    ColorId(ColorCombinationMaybe),
    Type(&'s str),
    TypeNot(&'s str),
    CardStats(card_stats::Query),
    ManaCost(mana_cost::ManaCostCount::Query),
    Term(&'s str),
    NameExact(&'s str),
    NegTerm(&'s str),
    NotNameExact(&'s str),
}

impl<'s> DbQueryFieldParam<'s> {
    pub fn into_index_param(self) -> Option<DbQueryIndex<'s>> {
        match self {
            DbQueryFieldParam::Color(c) => Some(DbQueryIndex::Color(c)),
            DbQueryFieldParam::ColorId(c) => Some(DbQueryIndex::ColorId(c)),
            DbQueryFieldParam::Type(t) => Some(DbQueryIndex::Type(t)),
            DbQueryFieldParam::CardStats(s) => Some(DbQueryIndex::CardStats(s)),
            DbQueryFieldParam::ManaCost(m) => Some(DbQueryIndex::ManaCost(m)),
            DbQueryFieldParam::NameExact(n) => Some(DbQueryIndex::NameExact(n)),
            _ => None,
        }
    }
}

pub struct DbQuery<'s> {
    index: Option<DbQueryIndex<'s>>,
    tree: DbQueryTree<'s>,
}

pub enum DbQueryTree<'s> {
    And(Vec<DbQueryTree<'s>>),
    Or(Vec<DbQueryTree<'s>>),
    Term(DbQueryFieldParam<'s>),
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
            items.iter()
            .map(|q| tree_to_tree(&q.query, q.source_range.start, compile_errs))
            .collect::<Option<Vec<_>>>()?,
        ),
        SearchQueryTree::Or(items) => DbQueryTree::Or(
            items
                .iter()
                .map(|q| tree_to_tree(&q.query, q.source_range.start, compile_errs))
                .collect::<Option<Vec<_>>>()?,
        ),
        SearchQueryTree::Term(term) => DbQueryTree::Term(term_to_field(
            term,
            byte_index,
            compile_errs,
        )?),
    })
}

fn find_index_field<'q, 'c>(
    q: &'c SearchQueryTree<'q>,
    byte_index: usize,
    compile_errs: &'c impl MessageSink,
) -> Option<DbQueryIndex<'q>> {
    match q {
        //due to the way that the DB indices work, we can't really OR query on them
        SearchQueryTree::Or(items) => None,
        SearchQueryTree::And(items) => {
            //no nested AND lists because we'll've already flattened.
        },
        SearchQueryTree::Term(search_term) => {
            term_to_field(search_term, byte_index, compile_errs).and_then(|x| x.into_index_param())
        }
    }
}

pub fn term_to_field<'q, 'c>(
    term: &'c SearchTerm<'q>,
    byte_index: usize,
    compile_errs: &'c impl MessageSink,
) -> Option<DbQueryFieldParam<'q>> {
    match term {
        SearchTerm::Term(s) => Some(DbQueryFieldParam::Term(s)),
        SearchTerm::NegTerm(s) => Some(DbQueryFieldParam::NegTerm(s)),
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
    match k {
        "c" | "color" | "id" => {
            let color_specified = color_name(v, byte_index, compile_errs)?;
            todo!()
        }
        _ => todo!(),
    }
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
                    'g' => f.red = true,
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
