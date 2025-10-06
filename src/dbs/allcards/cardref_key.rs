use std::num::NonZeroUsize;

use minimal_storage::{
    bit_sections::BitSection,
    serialize_min::{DeserializeFromMinimal, SerializeMinimal},
};

use crate::data_model::{card::CardRef, oddities::StringishUsize};

pub fn card_ref_to_index(cr: &CardRef) -> u128 {
    let mut bytes = [0u8; u128::BITS as usize / 8usize];

    let mut bytes_write = &mut bytes[..];

    let mut fb = BitSection::new();
    fb.set_bit(0, cr.collector_number.is_usize());

    cr.set
        .as_str()
        .minimally_serialize(&mut bytes_write, fb)
        .expect("cardref's set code should be transformed into an index");

    match &cr.collector_number {
        StringishUsize::Number(n) => n.minimally_serialize(&mut bytes_write, ()),
        StringishUsize::String(s) => s
            .as_str()
            .minimally_serialize(&mut bytes_write, BitSection::new()),
    }
    .expect("cardref's collector number should be transformed into an index");

    cr.printing
        .map(|x| x.get())
        .unwrap_or(0)
        .minimally_serialize(&mut bytes_write, ())
        .expect("cardref's printing should be transformed into an index");

    u128::from_le_bytes(bytes)
}

pub fn index_to_card_ref(idx: u128) -> CardRef {
    let bytes = idx.to_le_bytes();
    let collector_number_is_usize = (bytes[0] & 0b1000_0000) > 0;
    let mut bytes_read = &bytes[..];

    let set = String::deserialize_minimal(&mut bytes_read, None)
        .expect("set code should be embedded in index");

    let collector_number = if collector_number_is_usize {
        usize::deserialize_minimal(&mut bytes_read, ()).map(StringishUsize::Number)
    } else {
        String::deserialize_minimal(&mut bytes_read, None).map(StringishUsize::String)
    };
    let collector_number = collector_number.expect("collector number should be embedded in index");

    let printing = usize::deserialize_minimal(&mut bytes_read, ())
        .map(NonZeroUsize::new)
        .expect("Printing number should be embedded in index");

    CardRef { set, collector_number, printing }
}
