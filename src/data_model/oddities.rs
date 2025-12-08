// Code to help generically deal with odd behaviours

use std::{convert::Infallible, str::FromStr};

use minimal_storage::{
    bit_sections::BitSection,
    serialize_min::{DeserializeFromMinimal, ReadExtReadOne, SerializeMinimal},
};

///
/// A string that is normally formatted as `T::from_str` expects, but can rarely be in some other format
/// Because of Rust limitations on specialization, use `StringishUsize` for numbers.
pub enum Stringish<T> {
    Normal(T),
    Alternative(String),
}

impl<T: FromStr + SerializeMinimal> SerializeMinimal for Stringish<T> {
    type ExternalData<'s> = T::ExternalData<'s>;

    fn minimally_serialize<'a, 's: 'a, W: std::io::Write>(
        &'a self,
        write_to: &mut W,
        external_data: Self::ExternalData<'s>,
    ) -> std::io::Result<()> {
        match self {
            Stringish::Normal(normal) => {
                0u8.minimally_serialize(write_to, ())?;
                normal.minimally_serialize(write_to, external_data)
            }
            Stringish::Alternative(t) => t
                .as_str()
                .minimally_serialize(write_to, BitSection::from(0b10 << 6)),
        }
    }
}

impl<T: FromStr + DeserializeFromMinimal> DeserializeFromMinimal for Stringish<T> {
    type ExternalData<'d> = T::ExternalData<'d>;

    fn deserialize_minimal<'a, 'd: 'a, R: std::io::Read>(
        from: &'a mut R,
        external_data: Self::ExternalData<'d>,
    ) -> Result<Self, std::io::Error> {
        let first_byte = ReadExtReadOne::read_one(from)?;

        let sig = first_byte >> 6;

        if sig == 0b10 {
            Ok(Stringish::Alternative(String::deserialize_minimal(
                from,
                first_byte.into(),
            )?))
        } else {
            Ok(Stringish::Normal(T::deserialize_minimal(
                from,
                external_data,
            )?))
        }
    }
}

///
/// A string that is normally a number, but can be something else.
/// This is a workaround for Rust's limits on specialization
#[derive(Debug, Clone)]
pub enum StringishUsize {
    Number(usize),
    String(String),
}
impl StringishUsize {
    pub fn is_usize(&self) -> bool {
        match self {
            StringishUsize::Number(_) => true,
            StringishUsize::String(_) => false,
        }
    }
}

impl FromStr for StringishUsize {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse() {
            Ok(n) => Ok(Self::Number(n)),
            _ => Ok(Self::String(s.to_string())),
        }
    }
}

impl From<usize> for StringishUsize {
    fn from(value: usize) -> Self {
        Self::Number(value)
    }
}

impl From<String> for StringishUsize {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl SerializeMinimal for StringishUsize {
    type ExternalData<'s> = ();

    fn minimally_serialize<'a, 's: 'a, W: std::io::Write>(
        &'a self,
        write_to: &mut W,
        external_data: Self::ExternalData<'s>,
    ) -> std::io::Result<()> {
        match self {
            StringishUsize::Number(normal) => {
                let bits_to_put_in_first_byte = (*normal & 0b111_111) as u8;
                let rest_of_bits = *normal >> 6;

                let first_byte_is_end_flag = (rest_of_bits == 0) as u8;
                let variant_flag = 0b0;

                let first_byte =
                    (variant_flag << 7) | (first_byte_is_end_flag << 6) | bits_to_put_in_first_byte;

                first_byte.minimally_serialize(write_to, ())?;

                if rest_of_bits != 0 {
                    rest_of_bits.minimally_serialize(write_to, external_data)
                } else {
                    Ok(())
                }
            }
            StringishUsize::String(t) => t
                .as_str()
                .minimally_serialize(write_to, BitSection::from(0b1 << 7)),
        }
    }
}

impl DeserializeFromMinimal for StringishUsize {
    type ExternalData<'d> = ();

    fn deserialize_minimal<'a, 'd: 'a, R: std::io::Read>(
        from: &'a mut R,
        _external_data: Self::ExternalData<'d>,
    ) -> Result<Self, std::io::Error> {
        let first_byte = ReadExtReadOne::read_one(from)?;

        let variant_flag = first_byte >> 7;

        if variant_flag == 0b1 {
            Ok(StringishUsize::String(String::deserialize_minimal(
                from,
                first_byte.into(),
            )?))
        } else {
            let first_byte_is_end = ((first_byte >> 6) & 0b1) != 0;
            let bits_from_first_byte = first_byte & 0b111_111;

            if first_byte_is_end {
                return Ok(StringishUsize::Number(bits_from_first_byte as usize));
            }

            let rest_of_bits = usize::deserialize_minimal(from, ())?;

            let number = (rest_of_bits << 6) | (bits_from_first_byte as usize);

            Ok(StringishUsize::Number(number))
        }
    }
}
