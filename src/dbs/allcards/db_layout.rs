use crate::{
    data_model::card::ColorCombination,
    dbs::{
        allcards::{Card, DBTree},
        indexes::mana_cost::{self, ManaCostCount},
    },
};
use minimal_storage::{
    multitype_paged_storage::{MultitypePagedStorage, StoragePage, StoreByPage},
    paged_storage::PageId,
    serialize_min::{DeserializeFromMinimal, SerializeMinimal},
};

macro_rules! layout_all_cards_db {
    ( $($index_name:ident : $index_type:ty : $index_dim:literal dimensional $(,)? )* ) => {
        pub struct AllCardsDbLayout {
            pub cards_page: PageId<{ tree::PAGE_SIZE }>,

            $( pub $index_name: PageId<{ tree::PAGE_SIZE }>, )*
        }

        impl SerializeMinimal for AllCardsDbLayout {
            type ExternalData<'s> = ();

            fn minimally_serialize<'a, 's: 'a, W: std::io::Write>(
                &'a self,
                write_to: &mut W,
                _external_data: Self::ExternalData<'s>,
            ) -> std::io::Result<()> {
                self.cards_page.minimally_serialize(write_to, ())?;

                $( self.$index_name.minimally_serialize(write_to, ())?; )*

                Ok(())
            }
        }

        impl DeserializeFromMinimal for AllCardsDbLayout {
            type ExternalData<'d> = ();

            fn deserialize_minimal<'a, 'd: 'a, R: std::io::Read>(
                from: &'a mut R,
                _external_data: Self::ExternalData<'d>,
            ) -> Result<Self, std::io::Error> {
                let cards_page = PageId::deserialize_minimal(from, ())?;

                $( let $index_name = PageId::deserialize_minimal(from, ())?; )*

                Ok(Self {
                    cards_page,
                    $($index_name),*
                })
            }
        }
        pub struct AllCardsDb {
            pub(super) cards: DBTree<1, u128, Card>,

            $( pub(super) $index_name: DBTree<$index_dim, $index_type, u128>, )*
        }
        pub fn initialize_or_deserialize_db_layout(storage: &MultitypePagedStorage<{tree::PAGE_SIZE}, std::fs::File>) -> std::io::Result<AllCardsDb> {
            let known_layout_page_id = unsafe { PageId::from_index(std::num::NonZero::new(1).unwrap()) } ;
            match StoreByPage::<AllCardsDbLayout>::get(storage, &known_layout_page_id, ()) {
            Some(layout) => {
                let layout_read = layout.read();

                let cards = tree::sparse::open_storage(
                    u128::MIN..=u128::MAX,
                    storage,
                    Some(layout_read.cards_page),
                );
                use tree::tree_traits::MultidimensionalParent;

                $( let $index_name = tree::sparse::open_storage(<$index_type as tree::tree_traits::MultidimensionalKey<$index_dim>>::Parent::UNIVERSE, storage, Some(layout_read.$index_name)); )*

                Ok(AllCardsDb { cards, $( $index_name, )* })
            }
            None => {
                let mut db_swap = None::<AllCardsDb>;
                let layout_id = storage.new_page_with(|| {
                    let cards = tree::sparse::open_storage(u128::MIN..=u128::MAX, storage, None);
                    let cards_page = cards.root_page_id();

                    use tree::tree_traits::MultidimensionalParent;

                    $(
                                                    //just reusing the known_layout_page_id to have something to put there for now. It will be overwritten
                                                    //in the next statement.
                        let mut $index_name = (tree::sparse::open_storage(<$index_type as tree::tree_traits::MultidimensionalKey<$index_dim>>::Parent::UNIVERSE, storage, None), known_layout_page_id);
                        $index_name.1 = $index_name.0.root_page_id();
                    )*

                    db_swap = Some(AllCardsDb { cards, $( $index_name: $index_name.0, )* });

                    AllCardsDbLayout { cards_page, $( $index_name: $index_name.1, )* }
                });

                debug_assert_eq!(layout_id, known_layout_page_id);

                Ok(db_swap.unwrap())
            }
        }
        }
    };
}

layout_all_cards_db! {
    color: ColorCombination: 6 dimensional,
    color_id: ColorCombination:  6 dimensional,
    mana_cost: ManaCostCount::Key: 12 dimensional,
    //todo: implement indexes for supertypes,
    //float (mana_value), rarity
}
