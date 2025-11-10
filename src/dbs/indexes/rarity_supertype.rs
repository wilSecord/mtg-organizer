use crate::{data_model::card::{Rarity, Supertype}, dbs::indexes::helpers::make_index_types};

make_index_types! {
    key rarity {
        rarity: u8,
    }
}

impl rarity::Key {
    pub fn new(r: Rarity) -> Self {
        Self {
            rarity: match r {
                Rarity::Common => 0,
                Rarity::Uncommon => 1,
                Rarity::Rare => 2,
                Rarity::Mythic => 3,
                Rarity::Special => 4,
            }
        }
    }
}

make_index_types! {
    key supertype {
        supertype: u8,
    }
}

impl supertype::Key {
    pub fn new(r: Supertype) -> Self {
        Self {
            supertype: match r {
                Supertype::Basic => 0,
                Supertype::Legendary => 1,
                Supertype::Ongoing => 2,
                Supertype::Snow => 3,
                Supertype::World => 4,
                Supertype::Elite => 5,
                Supertype::Host => 6,
            }
        }
    }
}