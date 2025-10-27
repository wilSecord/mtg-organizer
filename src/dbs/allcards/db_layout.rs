use minimal_storage::{paged_storage::PageId, serialize_min::{DeserializeFromMinimal, SerializeMinimal}};

pub struct AllCardsDbLayout {
    pub cards_page: PageId<{tree::PAGE_SIZE}>
}

impl SerializeMinimal for AllCardsDbLayout {
    type ExternalData<'s> = ();

    fn minimally_serialize<'a, 's: 'a, W: std::io::Write>(
        &'a self,
        write_to: &mut W,
        external_data: Self::ExternalData<'s>,
    ) -> std::io::Result<()> {
        self.cards_page.minimally_serialize(write_to, ())?;
        Ok(())
    }
}

impl DeserializeFromMinimal for AllCardsDbLayout {
    type ExternalData<'d> = ();

    fn deserialize_minimal<'a, 'd: 'a, R: std::io::Read>(
            from: &'a mut R,
            external_data: Self::ExternalData<'d>,
        ) -> Result<Self, std::io::Error> {
        let cards_page = PageId::deserialize_minimal(from, ())?;

        Ok(Self {
            cards_page
        })
    }
}