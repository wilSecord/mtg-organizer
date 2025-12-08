use std::{
    result,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender},
    },
};

use nucleo_matcher::{
    Config, Matcher,
    pattern::{CaseMatching, Normalization, Pattern},
};

use crate::{
    dbs::allcards::AllCardsDb,
    query::{
        compile::build_search_query,
        err_warn_support::{self, Message, MessageSink},
    },
};

pub fn start_query_running_background_threads(
    db: Arc<AllCardsDb>,
) -> (Sender<String>, Receiver<(Option<Message>, Vec<String>)>) {
    let (tx_query, rx_query) = std::sync::mpsc::channel::<String>();
    let (tx_results, rx_results) = std::sync::mpsc::channel();

    let mut matcher = Matcher::new(Config::DEFAULT);

    std::thread::spawn(move || {
        loop {
            let Ok(search) = rx_query.recv() else {
                break;
            };

            let results = get_results(search.as_str(), &mut matcher, &db);

            tx_results.send(results).unwrap();
        }
    });

    (tx_query, rx_results)
}

fn get_results(
    search: &str,
    matcher: &mut Matcher,
    db: &AllCardsDb,
) -> (Option<Message>, Vec<String>) {
    struct ErrLineMessage<'s>(Mutex<&'s mut Option<Message>>);
    impl MessageSink for ErrLineMessage<'_> {
        fn send(&self, msg: Message) {
            **self.0.lock().unwrap() = Some(msg);
        }
    }

    let mut message = None;
    let errors = ErrLineMessage(Mutex::new(&mut message));

    let query = build_search_query(&search, &errors);

    let results = match query {
        Ok(query) => query.query_db(&db).map(|x| x.name).collect(),
        Err(simple_search) => Pattern::parse(
            simple_search.as_str(),
            CaseMatching::Ignore,
            Normalization::Smart,
        )
        .match_list(db.all_cards().map(|x| x.name), matcher)
        .into_iter()
        .map(|x| x.0.to_owned())
        .collect(),
    };

    (message, results)
}
