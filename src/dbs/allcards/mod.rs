use std::{fs::File, path::Path, sync::Arc, u128};

use minimal_storage::{
    multitype_paged_storage::{MultitypePagedStorage, SingleTypeView, StoragePage, StoreByPage},
    paged_storage::{Page, PageId},
    pooled_storage::Filelike,
};
use tree::sparse::structure::{Inner, Root};

use crate::{
    data_model::card::{Card, CardRef},
    dbs::allcards::{cardref_key::card_ref_to_index, db_layout::AllCardsDbLayout},
};

pub mod cardref_key;
mod db_layout;

type DBTree<Key, Value> = tree::sparse::StoredTree<
    1,
    8000,
    Key,
    Value,
    Page<{ tree::PAGE_SIZE }, Root<1, 8000, u128, Card>, std::fs::File>,
    SingleTypeView<{ tree::PAGE_SIZE }, std::fs::File, Inner<1, 8000, u128, Card>>,
>;

pub struct AllCardsDb {
    cards: DBTree<u128, Card>,
}

impl AllCardsDb {
    pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = std::fs::File::options()
            .create(true)
            .append(false)
            .read(true)
            .write(true)
            .open(path)?;

        let storage = MultitypePagedStorage::open(file);

        //the layout is ALWAYS stored at page #1
        match StoreByPage::<AllCardsDbLayout>::get(&storage, &PageId::new(1), ()) {
            Some(layout) => {
                let layout_read = layout.read();

                let cards = tree::sparse::open_storage(
                    u128::MIN..=u128::MAX,
                    &storage,
                    Some(layout_read.cards_page),
                );

                Ok(AllCardsDb { cards })
            }
            None => {
                let mut db_swap = None::<Self>;
                let layout_id = storage.new_page_with(|| {
                    let cards = tree::sparse::open_storage(u128::MIN..=u128::MAX, &storage, None);
                    let cards_page = cards.root_page_id();

                    db_swap = Some(AllCardsDb { cards });

                    AllCardsDbLayout { cards_page }
                });

                Ok(db_swap.unwrap())
            }
        }
    }

    pub fn add(&self, cardref: &CardRef, card: Card) {
        let id = card_ref_to_index(cardref);

        self.cards.insert(id, card);
    }
}

mod build {
    #[test]
    fn make_all_cards_db() {}
}
