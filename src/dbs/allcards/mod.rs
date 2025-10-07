use std::{fs::File, path::Path};

use minimal_storage::multitype_paged_storage::MultitypePagedStorage;
use tree::{sparse::structure::StoredTree, tree_traits::MultidimensionalKey};

use crate::data_model::card::{Card, CardRef};

pub mod cardref_key;

pub struct AllCardsDb {
    inner: StoredTree<1, 8000, u128, Card>
}


mod build {
    #[test]
    fn make_all_cards_db() {

    }
}