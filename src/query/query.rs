use std::sync::{mpsc::{Receiver, Sender}, Arc};

use crate::{data_model::card::Card, dbs::allcards::AllCardsDb, query::{err_warn_support::MessageSink, parse::{parse, parse_str, SearchQuery, SearchQueryTree}}};

struct DbQuerier {
    card_db: Arc<AllCardsDb>,
    current_results: Vec<Card>,
    update_pipe: Sender<String>
}

impl DbQuerier {
    pub fn new(card_db: Arc<AllCardsDb>) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        Self {
            card_db,
            current_results: Vec::new(),
            update_pipe: tx
        }
    }
}

fn start_update_loop(card_db: Arc<AllCardsDb>, search_query_update: Receiver<String>, on_update: Sender<()>) {
    let card_db = Arc::clone(&card_db);

    std::thread::spawn(move || {
        for search_query in search_query_update.iter() {
            let q = todo!();
        }
    });
}

pub fn search_card_database<'a>(card_db: &'a AllCardsDb, query: Option<SearchQuery<'a>>, msgs: impl MessageSink + 'a) -> impl Iterator<Item = Card> {

    query.into_iter().flat_map(|query| card_db.all_cards().filter(move |card| query.naive_matches_card(card)))
}