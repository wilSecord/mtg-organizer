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


    pub fn query_type<'a>(
        &'a self,
        q: &'a LongestPrefixMatch,
    ) -> impl Iterator<Item = impl AsRef<Card> + 'a> + 'a {
        self.types
            .find_entries_in_box(&q)
            .flat_map(|(slp, x)| {
                self.cards.get_readref(&x)
            })
    }

    pub fn query_type_for_id<'a>(
        &'a self,
        q: &'a LongestPrefixMatch,
    ) -> impl Iterator<Item = (StringPrefix, CardDbId)> + 'a {
        self.types
            .find_entries_in_box(&q)
            .map(|(c, id)| (c, CardDbId(id)))
    }

    pub fn query_color<'a>(
        &'a self,
        color: &'a ColorCombinationMaybe,
    ) -> impl Iterator<Item = impl AsRef<Card> + 'a> + 'a {
        self.color
            .find_items_in_box(&color)
            .flat_map(|x| self.cards.get_readref(&x))
    }
    pub fn query_mana<'a>(
        &'a self,
        query: &'a ManaCostCount::Query,
    ) -> impl Iterator<Item = impl AsRef<Card> + 'a> + 'a {
        self.mana_cost
            .find_items_in_box(&query)
            .flat_map(|x| self.cards.get_readref(&x))
    }

    pub fn all_cards(&self) -> impl Iterator<Item = Card> {
        self.cards.find_items_in_box(&(u128::MIN..=u128::MAX))
    }

    pub fn fulltext_search(
        &self,
        search: &str,
        min_match_proportion: f64,
    ) -> std::collections::BinaryHeap<impl Deref<Target = CardDbId> + Ord + 'static> {
        let mut intersect = BTreeMap::new();

        let mut total_trigrams = 0;

        for trigram in string_trigrams(0, &search) {
            let query = trigram::Query::any_field(&trigram);

            total_trigrams += 1;

            for doc_id in self.fulltext.find_items_in_box(&query) {
                intersect.entry(doc_id).or_insert(0).add_assign(1usize);
            }
        }

        if total_trigrams == 0 {
            return BinaryHeap::new();
        }

        let trigram_min_to_match = ((total_trigrams as f64) * min_match_proportion) as usize;

        struct OrderByFirst(f64, CardDbId);
        impl Deref for OrderByFirst {
            type Target = CardDbId;

            fn deref(&self) -> &Self::Target {
                &self.1
            }
        }
        impl PartialEq for OrderByFirst {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }
        impl Eq for OrderByFirst {}
        impl PartialOrd for OrderByFirst {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(&other))
            }
        }
        impl Ord for OrderByFirst {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.0.partial_cmp(&other.0).unwrap()
            }
        }

        intersect
            .into_iter()
            .filter(move |(_, v)| *v >= trigram_min_to_match)
            .map(|(id, count)| OrderByFirst(count as f64 / total_trigrams as f64, CardDbId(id)))
            .collect()
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

        for trigram in string_trigrams(0, &card.oracle_text) {
            self.fulltext.insert(trigram, id);
        }

        for trigram in string_trigrams(1, &card.name) {
            self.fulltext.insert(trigram, id);
        }

        for typ in card.types.iter() {
            let pkey = StringPrefix::new_prefix(typ.as_str().to_ascii_lowercase());
            self.types.insert(pkey, id);
        }

        for typ in card.subtypes.iter() {
            if(typ == "Adventure") {
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
