use std::{borrow::Cow, cmp::Ordering, fmt::Debug, io::Write, u128};

use minimal_storage::{
    serialize_fast::MinimalSerdeFast,
    serialize_min::{DeserializeFromMinimal, SerializeMinimal},
};
use tree::{
    sparse::SparseKey,
    tree_traits::{MaxValue, MinValue, MultidimensionalKey, MultidimensionalParent, Zero},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LongestPrefixMatch {
    bitlen: usize,
    bitbuf: u128,
}

#[derive(Clone, Copy, PartialOrd, PartialEq, Ord, Eq)]
pub struct StringPrefix {
    s: u128,
}

impl std::fmt::Debug for StringPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f
            .debug_struct("StringPrefix")
            .field("s", &self.s)
            .finish()
    }
}

impl std::fmt::Debug for LongestPrefixMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("LongestPrefixMatch");

        s.field("bitlen", &self.bitlen);
        s.field("bitbuf", &self.bitbuf);

        let buf_bytes = self.bitbuf.to_be_bytes();
        if let Ok(as_str) = str::from_utf8(&buf_bytes[..(self.bitlen / 8)]) {
            s.field("[as str]", &as_str);
        }

        s.finish()
    }
}

impl DeserializeFromMinimal for StringPrefix {
    type ExternalData<'d> = ();

    fn deserialize_minimal<'a, 'd: 'a, R: std::io::Read>(
        from: &'a mut R,
        external_data: Self::ExternalData<'d>,
    ) -> Result<Self, std::io::Error> {
        Self::fast_deserialize_minimal(from, external_data)
    }
}

impl SerializeMinimal for StringPrefix {
    type ExternalData<'s> = ();

    fn minimally_serialize<'a, 's: 'a, W: std::io::Write>(
        &'a self,
        write_to: &mut W,
        external_data: Self::ExternalData<'s>,
    ) -> std::io::Result<()> {
        self.fast_minimally_serialize(write_to, external_data)
    }
}

impl DeserializeFromMinimal for LongestPrefixMatch {
    type ExternalData<'d> = ();

    fn deserialize_minimal<'a, 'd: 'a, R: std::io::Read>(
        from: &'a mut R,
        external_data: Self::ExternalData<'d>,
    ) -> Result<Self, std::io::Error> {
        Self::fast_deserialize_minimal(from, external_data)
    }
}

impl SerializeMinimal for LongestPrefixMatch {
    type ExternalData<'s> = ();

    fn minimally_serialize<'a, 's: 'a, W: std::io::Write>(
        &'a self,
        write_to: &mut W,
        external_data: Self::ExternalData<'s>,
    ) -> std::io::Result<()> {
        self.fast_minimally_serialize(write_to, external_data)
    }
}

impl MinimalSerdeFast for StringPrefix {
    fn fast_minimally_serialize<'a, 's: 'a, W: std::io::Write>(
        &'a self,
        write_to: &mut W,
        _external_data: <Self as SerializeMinimal>::ExternalData<'s>,
    ) -> std::io::Result<()> {
        self.s.fast_minimally_serialize(write_to, ())
    }

    fn fast_deserialize_minimal<'a, 'd: 'a, R: std::io::Read>(
        from: &'a mut R,
        _external_data: <Self as minimal_storage::serialize_min::DeserializeFromMinimal>::ExternalData<'d>,
    ) -> Result<Self, std::io::Error> {
        u128::fast_deserialize_minimal(from, ()).map(|s| Self { s })
    }

    fn fast_seek_after<R: std::io::Read>(from: &mut R) -> std::io::Result<()> {
        u128::fast_seek_after(from)
    }
}



impl MinimalSerdeFast for LongestPrefixMatch {
    fn fast_minimally_serialize<'a, 's: 'a, W: std::io::Write>(
        &'a self,
        write_to: &mut W,
        _external_data: <Self as SerializeMinimal>::ExternalData<'s>,
    ) -> std::io::Result<()> {
        self.bitlen.minimally_serialize(write_to, ())?;
        //if the bit-length is 0, then early return because otherwise,
        //the shift will be 128. This is an overflow on shifting, since
        //it's shifting a 128-bit type by >= 128 bits. Therefore, imply
        //bitbuf == 0 for bitlen == 0
        if self.bitlen == 0 {
            return Ok(());
        }

        let shift = u128::BITS as usize - self.bitlen;

        debug_assert_eq!(self.bitbuf, (self.bitbuf >> shift) << shift);

        (self.bitbuf >> shift).minimally_serialize(write_to, ())
    }

    fn fast_deserialize_minimal<'a, 'd: 'a, R: std::io::Read>(
        from: &'a mut R,
        _external_data: <Self as minimal_storage::serialize_min::DeserializeFromMinimal>::ExternalData<'d>,
    ) -> Result<Self, std::io::Error> {
        let bitlen = usize::deserialize_minimal(from, ())?;

        if bitlen == 0 {
            return Ok(Self { bitlen, bitbuf: 0 });
        }

        let mut bitbuf = u128::deserialize_minimal(from, ())?;
        let shift = u128::BITS as usize - bitlen;

        bitbuf <<= shift;

        Ok(Self { bitlen, bitbuf })
    }

    fn fast_seek_after<R: std::io::Read>(from: &mut R) -> std::io::Result<()> {
        Self::fast_deserialize_minimal(from, ()).map(|_| ())
    }
}



impl MultidimensionalParent<1> for LongestPrefixMatch {
    type DimensionEnum = ();

    const UNIVERSE: Self = Self {
        bitlen: 0,
        bitbuf: 0,
    };

    fn contains(&self, child: &Self) -> bool {
        //if the parent is more specific, then we're not inside it.
        if self.bitlen > child.bitlen {
            return false;
        }

        //if the parent's length is 0, then it would be
        //an overflow on shifting. just autoreturn true.
        if self.bitlen == 0 {
            return true;
        }

        let shift = u128::BITS as usize - self.bitlen;

        (child.bitbuf >> shift) == (self.bitbuf >> shift)
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

impl MultidimensionalKey<1> for StringPrefix {
    type Parent = LongestPrefixMatch;

    type DeltaFromParent = u128;

    type DeltaFromSelfAsChild = u128;

    fn is_contained_in(&self, parent: &Self::Parent) -> bool {
        //if the parent's length is 0, then it would be
        //an overflow on shifting. just autoreturn true.
        if parent.bitlen == 0 {
            return true;
        }

        let shift = u128::BITS as usize - parent.bitlen;

        (self.s >> shift) == (parent.bitbuf >> shift)
    }

    fn delta_from_parent(&self, parent: &Self::Parent) -> Self::DeltaFromParent {
        let delta_amnt = parent.bitbuf ^ self.s;

        delta_amnt
    }

    fn apply_delta_from_parent(delta: &Self::DeltaFromParent, parent: &Self::Parent) -> Self {
        Self {
            s: delta | parent.bitbuf
        }
    }

    fn smallest_key_in(parent: &Self::Parent) -> Self {
        let shift = 128 - parent.bitlen;
        if shift == 128 {
            return Self { s: 0 };
        }

        let s = (parent.bitbuf >> shift) << shift;

        Self { s }
    }

    fn largest_key_in(parent: &Self::Parent) -> Self {
        let shift = 128 - parent.bitlen;
        if shift == 128 {
            return Self { s: u128::MAX };
        }

        let mask = ((!0u128) >> shift) << shift;

        let s = parent.bitbuf | mask;

        Self { s }
    }

    fn delta_from_self(
        finl: &Self::DeltaFromParent,
        initil: &Self::DeltaFromParent,
    ) -> Self::DeltaFromSelfAsChild {
        finl - initil
    }

    fn apply_delta_from_self(
        delta: &Self::DeltaFromSelfAsChild,
        initial: &Self::DeltaFromParent,
    ) -> Self::DeltaFromParent {
        initial + delta
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

impl StringPrefix {
    pub fn new_prefix<S: std::borrow::Borrow<str>>(s: S) -> Self {
        let mut write = vec![0; u128::BITS as usize / 8];
        let write_len = write.len();

        let s_str: &str = s.borrow();

        (&mut write[..])
            .write_all(&s_str.as_bytes()[0..std::cmp::min(s_str.len(), write_len)])
            .unwrap();

        Self {
            s: u128::from_be_bytes(write.try_into().unwrap())
        }
    }
}

impl LongestPrefixMatch {
    const MAX_BYTES: usize = u128::BITS as usize / 8;

    fn push_bit(&mut self, new_bit_value: bool) {
        self.bitlen += 1;
        if self.bitlen >= 128 {
            self.bitlen = 128;
        }
        self.bitbuf |= (new_bit_value as u128) << (128 - self.bitlen);
    }
    pub fn new<S: std::borrow::Borrow<str>>(s: S) -> Result<Self, StringTooLongErr> {
        let mut write = vec![0; u128::BITS as usize / 8];

        let s_bytes: &[u8] = s.borrow().as_bytes();

        (&mut write[..])
            .write_all(s_bytes)
            .map_err(|_| StringTooLongErr)?;

        Ok(Self {
            bitbuf: u128::from_be_bytes(write.try_into().unwrap()),
            bitlen: s_bytes.len() * 8,
        })
    }

    pub fn new_prefix<S: std::borrow::Borrow<str>>(s: S) -> Self {
        let s_str: &str = s.borrow();

        let slice = if s_str.as_bytes().len() > Self::MAX_BYTES {
            &s_str[0..Self::MAX_BYTES]
        } else {
            s_str
        };

        Self::new(slice)
            .expect("Should have checked to ensure that StringTooLongErr doesn't happen")
    }
}

impl MinValue for StringPrefix {
    const MIN: Self = Self {
        s: u128::MIN
    };
}
impl MaxValue for StringPrefix {
    const MAX: Self = Self {
        s: u128::MAX
    };
}

impl Default for StringPrefix {
    fn default() -> Self {
        Self {
            s: 0
        }
    }
}

#[cfg(test)]
mod test {
    use tree::tree_traits::{MultidimensionalKey, MultidimensionalParent};

    use crate::dbs::indexes::string_lpm::{LongestPrefixMatch, StringPrefix};

    #[test]
    fn anything_inside_universe() {
        assert!(
            StringPrefix::new_prefix("Hello!")
                .is_contained_in(&LongestPrefixMatch::UNIVERSE)
        );
        assert!(
            StringPrefix::new_prefix("Apes").is_contained_in(&LongestPrefixMatch::UNIVERSE)
        );
        assert!(
            StringPrefix::new_prefix("").is_contained_in(&LongestPrefixMatch::UNIVERSE)
        );
    }

    #[test]
    fn inside_prefix() {
        assert!(
            StringPrefix::new_prefix("state")
                .is_contained_in(&LongestPrefixMatch::new_prefix("s"))
        );
        assert!(
            !StringPrefix::new_prefix("tate")
                .is_contained_in(&LongestPrefixMatch::new_prefix("s"))
        );

        assert!(
            StringPrefix::new_prefix("PORK!! MY FAVOURITE")
                .is_contained_in(&LongestPrefixMatch::new_prefix("PORK!!"))
        );
        assert!(
            StringPrefix::new_prefix("BETRAYAL OF THE FOREMOST KIND").is_contained_in(
                &LongestPrefixMatch::new_prefix("BETRAYAL OF THE FOREMOST KIND")
            )
        );

        //we can't tell after 16 chars
        assert!(
            StringPrefix::new_prefix("BETRAYAL OF THE FOREMOST KIND").is_contained_in(
                &LongestPrefixMatch::new_prefix("BETRAYAL OF THE WORST KIND")
            )
        );
    }

    #[test]
    fn splitting() {
        let a = LongestPrefixMatch::UNIVERSE
            .split_evenly_on_dimension(&())
            .1;
        assert_eq!(
            a,
            LongestPrefixMatch {
                bitlen: 1,
                bitbuf: 1 << 127
            }
        );

        let b = a.split_evenly_on_dimension(&()).0;

        assert_eq!(
            b,
            LongestPrefixMatch {
                bitlen: 2,
                bitbuf: 0b10 << 126
            }
        );

        let c = b.split_evenly_on_dimension(&()).1;

        assert_eq!(
            c,
            LongestPrefixMatch {
                bitlen: 3,
                bitbuf: 0b101 << 125
            }
        );
    }
}
