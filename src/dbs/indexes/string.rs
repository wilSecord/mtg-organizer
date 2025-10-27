use std::{borrow::Cow, cmp::Ordering};

use minimal_storage::{
    serialize_fast::MinimalSerdeFast,
    serialize_min::{DeserializeFromMinimal, SerializeMinimal},
};
use tree::{
    sparse::SparseKey,
    tree_traits::{MultidimensionalKey, MultidimensionalParent, Zero},
};

#[derive(Debug, Clone, Copy)]
pub struct StringLongestPrefix {
    bitlen: usize,
    bitbuf: u128
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
        write_to.write_all(&self.bitbuf[0..(self.bitlen / 8)])
    }

    fn fast_deserialize_minimal<'a, 'd: 'a, R: std::io::Read>(
        from: &'a mut R,
        _external_data: <Self as minimal_storage::serialize_min::DeserializeFromMinimal>::ExternalData<'d>,
    ) -> Result<Self, std::io::Error> {
        let bitlen = usize::deserialize_minimal(from, ())?;

        let mut bitbuf = [0u8; MAX_LEN_BYTES];
        from.read_exact(&mut bitbuf[..(bitlen / 8)])?;

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
        let equal_extent = std::cmp::min(self.bitlen, other.bitlen);

        for i in 0..equal_extent {
            let byte = i / 8;
            let bit = 7 - (i % 8);

            let self_bit = (self.bitbuf[byte] >> bit) & 1;
            let other_bit = (other.bitbuf[byte] >> bit) & 1;

            if self_bit == other_bit {
                continue;
            }

            if self_bit > other_bit {
                return Ordering::Greater;
            } else {
                return Ordering::Less;
            }
        }

        //if we've made it here, then they're equal up unto the prefix.
        //At this point, if they're of equal length, then they're the same
        //LPM.
        if self.bitlen == other.bitlen {
            return Ordering::Equal;
        }
        //otherwise, decide based on the next bit from the longer one.
        let last_bit_index = equal_extent - 1;
        let last_byte = last_bit_index / 8;
        let last_bit = 7 - (last_bit_index % 8);

        let (self_nextbit, other_nextbit) = if self.bitlen > other.bitlen {
            (Some((self.bitbuf[last_byte] >> last_bit) != 0), None)
        } else {
            (None, Some((other.bitbuf[last_byte] >> last_bit) != 0))
        };

        //we consider any shorter prefix to be exactly in the middle of all of its possible suffixes.
        match (self_nextbit, other_nextbit) {
            (None, None) => unreachable!(),
            (Some(_), Some(_)) => unreachable!(),

            (None, Some(true)) => Ordering::Less,
            (None, Some(false)) => Ordering::Greater,

            (Some(true), None) => Ordering::Greater,
            (Some(false), None) => Ordering::Less,
        }
    }
}

impl MultidimensionalParent<1> for StringLongestPrefix {
    type DimensionEnum = ();

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
        (
            l,r
        )
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

        for i in 0..parent.bitlen {
            let byte = i / 8;
            let bit = 7 - (i % 8);

            let self_bit = (self.bitbuf[byte] >> bit) & 1;
            let other_bit = (parent.bitbuf[byte] >> bit) & 1;

            if self_bit != other_bit {
                return false;
            }
        }

        return true;
    }

    fn delta_from_parent(&self, parent: &Self::Parent) -> Self::DeltaFromParent {
        let delta_len = self
            .bitlen
            .checked_sub(parent.bitlen)
            .expect("Children should be checked to be inside the parent!");

        let delta_amnt = 
    }

    fn apply_delta_from_parent(delta: &Self::DeltaFromParent, parent: &Self::Parent) -> Self {
        todo!()
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

impl StringLongestPrefix {
    fn push_bit(&mut self, new_bit_value: bool) {
        self.bitlen += 1;
        let bitlen = self.bitlen + 1;
        let bit = 7 - ((bitlen - 1) % 8);
        let byte = (bitlen - 1) / 8;

        //like how dividing an integer silently truncates, it's okay to silently
        //truncate at this level of detail. Any conversion from a String will
        //make sure that it's valid anyway
        if byte < MAX_LEN_BYTES {
            self.bitbuf[byte] |= (new_bit_value as u8) << bit;
        }
    }
}

impl Zero for StringLongestPrefix {
    fn zero() -> Self {
        Default::default()
    }
}

impl Default for StringLongestPrefix {
    fn default() -> Self {
        Self {
            bitlen: 0,
            bitbuf: [0; MAX_LEN_BYTES],
        }
    }
}
