use std::{
    collections::{BTreeMap, BinaryHeap},
    ops::{AddAssign, Deref},
    path::Path,
    u128,
};

use minimal_storage::{
    multitype_paged_storage::{MultitypePagedStorage, SingleTypeView},
    paged_storage::Page,
};
use tree::{
    sparse::structure::{Inner, Root},
    tree_traits::{MaxValue, MultidimensionalParent},
};

use crate::{
    data_model::card::{Card, CardRef},
    dbs::{
        allcards::cardref_key::card_ref_to_index,
        indexes::{
            color_combination::ColorCombinationMaybe,
            mana_cost::ManaCostCount,
            stats::card_stats,
            string_lpm::{LongestPrefixMatch, StringPrefix},
            string_trigram::{string_trigrams, trigram},
        },
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CardDbId(u128);

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

    pub fn query_type<'a>(&'a self, q: &'a LongestPrefixMatch) -> impl Iterator<Item = Card> + 'a {
        self.types
            .find_items_in_box(&q)
            .flat_map(|x| self.cards.get_owned(&x))
    }

    pub fn query_name<'a>(
        &'a self,
        query: &'a LongestPrefixMatch,
    ) -> impl Iterator<Item = Card> + 'a {
        self.card_names
            .find_items_in_box(&query)
            .flat_map(|x| self.cards.get_owned(&x))
    }

    pub fn query_color<'a>(
        &'a self,
        color: &'a ColorCombinationMaybe,
    ) -> impl Iterator<Item = Card> + 'a {
        self.color
            .find_items_in_box(&color)
            .flat_map(|x| self.cards.get_owned(&x))
    }
    pub fn query_color_id<'a>(
        &'a self,
        color_id: &'a ColorCombinationMaybe,
    ) -> impl Iterator<Item = Card> + 'a {
        self.color_id
            .find_items_in_box(&color_id)
            .flat_map(|x| self.cards.get_owned(&x))
    }
    pub fn query_mana<'a>(
        &'a self,
        query: &'a ManaCostCount::Query,
    ) -> impl Iterator<Item = Card> + 'a {
        self.mana_cost
            .find_items_in_box(&query)
            .flat_map(|x| self.cards.get_owned(&x))
    }
    pub fn query_stats<'a>(
        &'a self,
        query: &'a card_stats::Query,
    ) -> impl Iterator<Item = Card> + 'a {
        self.stats
            .find_items_in_box(&query)
            .flat_map(|x| self.cards.get_owned(&x))
    }

    pub fn all_cards(&self) -> impl Iterator<Item = Card> {
        self.cards.find_items_in_box(&(u128::MIN..=u128::MAX))
    }

    pub fn get_card(&self, card: CardDbId) -> Option<impl AsRef<Card>> {
        self.cards.get_readref(&card.0)
    }

    pub fn add(&self, cardref: &CardRef, card: Card) {
        let id = card_ref_to_index(cardref);
        let _increasing_idx = self
            .num_cards
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel);

        self.color.insert(card.color, id);
        self.color_id.insert(card.color_id, id);
        self.mana_cost
            .insert(ManaCostCount::Key::new(&card.mana_cost), id);

        self.card_names
            .insert(StringPrefix::new_prefix(&*card.name), id);

        for typ in card.types.iter() {
            let pkey = StringPrefix::new_prefix(typ.as_str().to_ascii_lowercase());
            self.types.insert(pkey, id);
        }

        for typ in card.subtypes.iter() {
            if (typ == "Adventure") {
                dbg!("yeah it's an adventure");
            }
            let pkey = StringPrefix::new_prefix(typ.as_str().to_ascii_lowercase());
            self.types.insert(pkey, id);
        }

        self.stats.insert(card_stats::Key::new(&card), id);

        self.cards.insert(id, card);
    }
}

mod build {
    #[test]
    fn make_all_cards_db() {}
}
