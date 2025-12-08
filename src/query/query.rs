use std::sync::{mpsc::{Receiver, Sender}, Arc, Mutex};

use nucleo_matcher::{pattern::{CaseMatching, Normalization, Pattern}, Config, Matcher};

use crate::{dbs::allcards::AllCardsDb, query::{compile::build_search_query, err_warn_support::{self, Message, MessageSink}}};

pub fn start_query_running_background_threads(db: Arc<AllCardsDb>) -> (Sender<String>, Receiver<(String, Vec<String>)>) {
    let (tx_query, rx_query) = std::sync::mpsc::channel::<String>();
    let (tx_results, rx_results) = std::sync::mpsc::channel();

    let mut matcher = Matcher::new(Config::DEFAULT);

    std::thread::spawn(move|| loop {
        let Ok(search) = rx_query.recv() else {
            break;
        };

        let mut messageline = String::new();
        let results = get_results(search.as_str(), &mut messageline, &mut matcher, &db);

        tx_results.send((search, results)).unwrap();
    });

    (tx_query, rx_results)
}

fn get_results(search: &str, messageline: &mut String, matcher: &mut Matcher, db: &AllCardsDb) -> Vec<String> {
        struct ErrLineMessage<'s>(Mutex<&'s mut String>);
        impl MessageSink for ErrLineMessage<'_> {
            fn send(&self, msg: Message) {
                **self.0.lock().unwrap() = msg.msg_content;
            }
        }

        messageline.truncate(0);

        let errors = ErrLineMessage(Mutex::new(messageline));

        let query = build_search_query(&search, &errors);

        match query {
            Ok(query) => return query.query_db(&db).map(|x| x.name).collect(),
            Err(simple_search) => {
                return Pattern::parse(
                    simple_search.as_str(),
                    CaseMatching::Ignore,
                    Normalization::Smart,
                )
                .match_list(db.all_cards().map(|x| x.name), matcher)
                .into_iter()
                .map(|x| x.0.to_owned())
                .collect();
            }
        }
    }