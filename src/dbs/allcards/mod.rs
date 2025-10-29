use std::{fs::File, path::Path, sync::Arc, u128};

use minimal_storage::{
    multitype_paged_storage::{MultitypePagedStorage, SingleTypeView, StoragePage, StoreByPage},
    paged_storage::{Page, PageId},
    pooled_storage::Filelike,
};
use tree::sparse::structure::{Inner, Root};

use crate::{
    data_model::card::{Card, CardRef},
    dbs::{
        allcards::cardref_key::card_ref_to_index, indexes::color_combination::ColorCombinationMaybe,
    },
};

pub mod cardref_key;
mod db_layout;

pub use db_layout::AllCardsDb;

type DBTree<const DIMENSIONS: usize, Key, Value> = tree::sparse::StoredTree<
    DIMENSIONS,
    8000,
    Key,
    Value,
    Page<{ tree::PAGE_SIZE }, Root<DIMENSIONS, 8000, Key, Value>, std::fs::File>,
    SingleTypeView<{ tree::PAGE_SIZE }, std::fs::File, Inner<DIMENSIONS, 8000, Key, Value>>,
>;

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
        db_layout::initialize_or_deserialize_db_layout(&storage)
    }

    pub fn query_color<'a>(&'a self, color: &'a ColorCombinationMaybe) -> impl Iterator<Item = impl AsRef<Card> + 'a> + 'a {
        self.color.find_items_in_box(&color).flat_map(|x| self.cards.get_readref(&x))
    }

    pub fn all_cards(&self) -> impl Iterator<Item = Card> {
        self.cards.find_items_in_box(&(u128::MIN..=u128::MAX))
    }
    
    pub fn add(&self, cardref: &CardRef, card: Card) {
        let id = card_ref_to_index(cardref);

        self.color.insert(card.color, id);
        self.color_id.insert(card.color_id, id);
        self.cards.insert(id, card);
    }
}

mod build {
    #[test]
    fn make_all_cards_db() {}
}
