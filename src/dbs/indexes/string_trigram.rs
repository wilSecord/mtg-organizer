use std::{borrow::Cow, cmp::Ordering, fmt::Debug};

use minimal_storage::{
    serialize_fast::MinimalSerdeFast,
    serialize_min::{DeserializeFromMinimal, SerializeMinimal},
};
use tree::{
    sparse::SparseKey,
    tree_traits::{MaxValue, MinValue, MultidimensionalKey, MultidimensionalParent, Zero},
};

use crate::dbs::indexes::helpers::make_index_types;

make_index_types! {
    key trigram {
        index: usize,
        #[fast]
        chars: u32
    }
}

#[inline]
fn abc_as_u32(abc: &[u8], field: u8) -> u32 {
    let mut res = 0u32;

    debug_assert!(abc.len() < 4);

    for byte in abc {
        res <<= 8;
        res |= byte.to_ascii_uppercase() as u32;
    }

    res <<= 8;
    res |= field as u32;

    res
}

impl trigram::Query {
    pub fn any_field(trigram: &trigram::Key) -> Self {
        let min_field = u8::MIN as u32;
        let max_field = u8::MAX as u32;

        let chars_blank_field = trigram.chars & !0xFF;

        Self {
            index: trigram.index..=MaxValue::MAX,
            chars: (chars_blank_field | min_field)..=(chars_blank_field | max_field),
        }
    }
}

pub fn string_trigrams(field: u8, s: &str) -> impl Iterator<Item = trigram::Key> {
    let bytes = s.as_bytes();
    let short_fallback_iter = (bytes.len() < 3)
        .then(|| trigram::Key {
            index: 0,
            chars: abc_as_u32(bytes, field),
        })
        .into_iter();

    let normal_case_iter = bytes
        .windows(3)
        .enumerate()
        .map(move |(i, window)| trigram::Key {
            index: i,
            chars: abc_as_u32(window, field),
        });

    return short_fallback_iter.chain(normal_case_iter);
}
