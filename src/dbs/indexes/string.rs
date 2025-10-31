use std::{borrow::Cow, cmp::Ordering, fmt::Debug};

use minimal_storage::{
    serialize_fast::MinimalSerdeFast,
    serialize_min::{DeserializeFromMinimal, SerializeMinimal},
};
use tree::{
    sparse::SparseKey,
    tree_traits::{MinValue, MultidimensionalKey, MultidimensionalParent, Zero},
};

#[derive(Debug, Clone, Copy)]
pub struct StringLongestPrefix {
    bitlen: usize,
    bitbuf: u128,
}

impl DeserializeFromMinimal for StringLongestPrefix {
    type ExternalData<'d> = ();

    fn deserialize_minimal<'a, 'd: 'a, R: std::io::Read>(
        from: &'a mut R,
        external_data: Self::ExternalData<'d>,
    ) -> Result<Self, std::io::Error> {
        Self::fast_deserialize_minimal(from, external_data)
    }
}

impl SerializeMinimal for StringLongestPrefix {
    type ExternalData<'s> = ();

    fn minimally_serialize<'a, 's: 'a, W: std::io::Write>(
        &'a self,
        write_to: &mut W,
        external_data: Self::ExternalData<'s>,
    ) -> std::io::Result<()> {
        self.fast_minimally_serialize(write_to, external_data)
    }
}

impl MinimalSerdeFast for StringLongestPrefix {
    fn fast_minimally_serialize<'a, 's: 'a, W: std::io::Write>(
        &'a self,
        write_to: &mut W,
        _external_data: <Self as SerializeMinimal>::ExternalData<'s>,
    ) -> std::io::Result<()> {
        self.bitlen.minimally_serialize(write_to, ())?;

        let shift = u128::BITS as usize - self.bitlen;

        (self.bitbuf >> shift).minimally_serialize(write_to, ())
    }

    fn fast_deserialize_minimal<'a, 'd: 'a, R: std::io::Read>(
        from: &'a mut R,
        _external_data: <Self as minimal_storage::serialize_min::DeserializeFromMinimal>::ExternalData<'d>,
    ) -> Result<Self, std::io::Error> {
        let bitlen = usize::deserialize_minimal(from, ())?;

        let mut bitbuf = u128::deserialize_minimal(from, ())?;
        let shift = u128::BITS as usize - bitlen;

        bitbuf <<= shift;

        Ok(Self { bitlen, bitbuf })
    }

    fn fast_seek_after<R: std::io::Read>(from: &mut R) -> std::io::Result<()> {
        todo!()
    }
}

impl PartialEq for StringLongestPrefix {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd for StringLongestPrefix {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Eq for StringLongestPrefix {}

impl Ord for StringLongestPrefix {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.bitbuf.cmp(&other.bitbuf)
    }
}

impl MultidimensionalParent<1> for StringLongestPrefix {
    type DimensionEnum = ();

    const UNIVERSE: Self = Self {
        bitlen: 0,
        bitbuf: 0,
    };

    fn contains(&self, child: &Self) -> bool {
        child.is_contained_in(self)
    }

    fn overlaps(&self, child: &Self) -> bool {
        //there's no way that it can overlap without being contained
        self.contains(child)
    }

    fn split_evenly_on_dimension(&self, _: &Self::DimensionEnum) -> (Self, Self) {
        let mut l = self.to_owned();
        let mut r = self.to_owned();
        l.push_bit(false);
        r.push_bit(true);
        (l, r)
    }
}

impl MultidimensionalKey<1> for StringLongestPrefix {
    type Parent = Self;

    type DeltaFromParent = Self;

    type DeltaFromSelfAsChild = Self;

    fn is_contained_in(&self, parent: &Self::Parent) -> bool {
        if self.bitlen > parent.bitlen {
            return false;
        }

        (self.bitbuf >> parent.bitlen) == (parent.bitbuf >> parent.bitlen)
    }

    fn delta_from_parent(&self, parent: &Self::Parent) -> Self::DeltaFromParent {
        let delta_len = self.bitlen - parent.bitlen;
        let delta_amnt = parent.bitbuf ^ self.bitbuf;

        Self {
            bitlen: delta_len,
            bitbuf: delta_amnt,
        }
    }

    fn apply_delta_from_parent(delta: &Self::DeltaFromParent, parent: &Self::Parent) -> Self {
        Self {
            bitlen: delta.bitlen + parent.bitlen,
            bitbuf: delta.bitbuf | parent.bitbuf,
        }
    }

    fn smallest_key_in(parent: &Self::Parent) -> Self {
        let mut sk = parent.to_owned();
        sk.push_bit(false);
        sk
    }

    fn largest_key_in(parent: &Self::Parent) -> Self {
        let mut sk = parent.to_owned();
        sk.push_bit(true);
        sk
    }

    fn delta_from_self(
        finl: &Self::DeltaFromParent,
        initil: &Self::DeltaFromParent,
    ) -> Self::DeltaFromSelfAsChild {
        Self::delta_from_parent(finl, initil)
    }

    fn apply_delta_from_self(
        delta: &Self::DeltaFromSelfAsChild,
        initial: &Self::DeltaFromParent,
    ) -> Self::DeltaFromParent {
        Self::apply_delta_from_parent(delta, initial)
    }
}

pub struct StringTooLongErr;

impl Debug for StringTooLongErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            "[String too long; strings in DB keys must be <= 16 bytes long after compression]",
        )
    }
}

impl StringLongestPrefix {
    fn push_bit(&mut self, new_bit_value: bool) {
        self.bitbuf |= (new_bit_value as u128) << self.bitlen;
        self.bitlen += 1;
    }
    pub fn new<S: std::borrow::Borrow<str>>(s: S) -> Result<Self, StringTooLongErr> {
        let mut write = Vec::new();

        const MAX_BYTES: usize = u128::BITS as usize / 8;

        s.borrow()
            .minimally_serialize(&mut &mut write[0..MAX_BYTES], 0.into())
            .map_err(|_| StringTooLongErr)?;

        let written_length = write.len();

        write.extend(std::iter::repeat_n(0u8, MAX_BYTES - write.len()));

        Ok(Self {
            bitbuf: u128::from_be_bytes(write.try_into().unwrap()),
            bitlen: written_length * 8,
        })
    }
}

impl MinValue for StringLongestPrefix {
    const MIN: Self = Self {
        bitlen: 0,
        bitbuf: 0,
    };
}

impl Default for StringLongestPrefix {
    fn default() -> Self {
        Self {
            bitlen: 0,
            bitbuf: 0,
        }
    }
}
