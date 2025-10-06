use std::io::ErrorKind;

use minimal_storage::{
    bit_sections::{self, BitSection, Byte},
    serialize_fast::MinimalSerdeFast,
    serialize_min::{DeserializeFromMinimal, ReadExtReadOne, SerializeMinimal},
};

use crate::data_model::card::{
    Card, Color, ColorCombination, ManaCost, ManaSymbol, ManaVariable,
    Rarity, Supertype,
};

impl MinimalSerdeFast for Card {
    fn fast_minimally_serialize<'a, 's: 'a, W: std::io::Write>(
        &'a self,
        write_to: &mut W,
        external_data: <Self as SerializeMinimal>::ExternalData<'s>,
    ) -> std::io::Result<()> {
        self.minimally_serialize(write_to, external_data)
    }

    fn fast_deserialize_minimal<'a, 'd: 'a, R: std::io::Read>(
        from: &'a mut R,
        external_data: <Self as DeserializeFromMinimal>::ExternalData<'d>,
    ) -> Result<Self, std::io::Error> {
        Self::deserialize_minimal(from, external_data)
    }

    fn fast_seek_after<R: std::io::Read>(from: &mut R) -> std::io::Result<()> {
        Self::deserialize_minimal(from, ())?;
        Ok(())
    }
}

impl DeserializeFromMinimal for Card {
    type ExternalData<'d> = ();

    fn deserialize_minimal<'a, 'd: 'a, R: std::io::Read>(
        from: &'a mut R,
        _: Self::ExternalData<'d>,
    ) -> Result<Self, std::io::Error> {
        let name_fb = from.read_one()?;
        let rarity = match name_fb >> 5 {
            0 => Rarity::Common,
            1 => Rarity::Uncommon,
            2 => Rarity::Rare,
            3 => Rarity::Mythic,
            4 => Rarity::Special,
            _ => return Err(ErrorKind::InvalidData.into()),
        };
        let name = String::deserialize_minimal(from, Some(name_fb))?;

        let mana_value = usize::deserialize_minimal(from, ())? as f64 * 4.0;

        let mana_cost = ManaCost::deserialize_minimal(from, ())?;

        let color = ColorCombination::deserialize_minimal(from, ())?;
        let color_id = ColorCombination::deserialize_minimal(from, ())?;

        let super_types = read_supertype_list(from)?;

        let types = Vec::<String>::deserialize_minimal(from, None)?;
        let subtypes = Vec::<String>::deserialize_minimal(from, None)?;
        let sets_released = Vec::<String>::deserialize_minimal(from, None)?;

        let game_changer_byte = from.read_one()?;
        let game_changer = game_changer_byte & 0b1000_0000 > 1;

        let oracle_text = String::deserialize_minimal(from, Some(game_changer_byte))?;

        let power = usize::deserialize_minimal(from, ())?;
        let toughness = usize::deserialize_minimal(from, ())?;
        let loyalty = usize::deserialize_minimal(from, ())?;
        let defense = usize::deserialize_minimal(from, ())?;

        Ok(Card {
            name,
            mana_cost,
            mana_value,
            color,
            color_id,
            super_types,
            types,
            subtypes,
            rarity,
            oracle_text,
            power,
            toughness,
            loyalty,
            defense,
            sets_released,
            game_changer,
        })
    }
}

impl SerializeMinimal for Card {
    type ExternalData<'s> = ();

    fn minimally_serialize<'a, 's: 'a, W: std::io::Write>(
        &'a self,
        write_to: &mut W,
        _: Self::ExternalData<'s>,
    ) -> std::io::Result<()> {
        let rarity: u8 = match self.rarity {
            Rarity::Common => 0,
            Rarity::Uncommon => 1,
            Rarity::Rare => 2,
            Rarity::Mythic => 3,
            Rarity::Special => 4,
        };
        debug_assert!(rarity & 0b111 == rarity);

        //the rarity is less than 3 bits, so we can stuff it in before the name
        self.name
            .as_str()
            .minimally_serialize(write_to, BitSection::from(rarity))?;

        //there should, AT LEAST, be quarter-mana. provided WotC doesn't do
        //something horrible
        let mana_value_int = self.mana_value * 4.0;
        debug_assert!(mana_value_int.fract() == 0.0);
        (mana_value_int as usize).minimally_serialize(write_to, ())?;
        self.mana_cost.minimally_serialize(write_to, ())?;

        self.color.minimally_serialize(write_to, ())?;
        self.color_id.minimally_serialize(write_to, ())?;

        write_supertype_list(&self.super_types, write_to)?;

        self.types.len().minimally_serialize(write_to, ())?;
        for t in &self.types {
            //TODO: make a string pool or smth to hold these in
            t.as_str().minimally_serialize(write_to, 0u8.into())?;
        }

        self.subtypes.len().minimally_serialize(write_to, ())?;
        for t in &self.subtypes {
            //TODO: make a string pool or smth to hold these in
            t.as_str().minimally_serialize(write_to, 0u8.into())?;
        }

        self.sets_released.len().minimally_serialize(write_to, ())?;
        for t in &self.sets_released {
            //TODO: make a string pool or smth to hold these in
            t.as_str().minimally_serialize(write_to, 0u8.into())?;
        }

        //put game_changer in the extra bits of the oracle text
        let game_changer_byte = (self.game_changer as u8) << 7;

        self.oracle_text
            .as_str()
            .minimally_serialize(write_to, game_changer_byte.into())?;

        self.power.minimally_serialize(write_to, ())?;
        self.toughness.minimally_serialize(write_to, ())?;
        self.loyalty.minimally_serialize(write_to, ())?;
        self.defense.minimally_serialize(write_to, ())?;

        Ok(())
    }
}

fn read_supertype_list(read_from: &mut impl std::io::Read) -> std::io::Result<Vec<Supertype>> {
    let mut vec = Vec::new();
    loop {
        let b = read_from.read_one()?;

        for nibble in [b >> 4, b & 0b1111] {
            vec.push(match nibble & 0b111 {
                0 => Supertype::Basic,
                1 => Supertype::Legendary,
                2 => Supertype::Ongoing,
                3 => Supertype::Snow,
                4 => Supertype::World,
                5 => Supertype::Elite,
                6 => Supertype::Host,
                _ => return Err(ErrorKind::InvalidData.into()),
            });
            if nibble & 0b1000 > 0 {
                return Ok(vec);
            }
        }
    }
}

fn write_supertype_list(
    super_types: &Vec<Supertype>,
    write_to: &mut impl std::io::Write,
) -> std::io::Result<()> {
    let total_itms = super_types.len();
    let mut itms_consumed = 0;
    for chunk in super_types.chunks(2) {
        let mut b = 0u8;
        for itm in chunk {
            itms_consumed += 1;
            let is_last = total_itms == itms_consumed;

            let itm_b = match itm {
                Supertype::Basic => 0,
                Supertype::Legendary => 1,
                Supertype::Ongoing => 2,
                Supertype::Snow => 3,
                Supertype::World => 4,
                Supertype::Elite => 5,
                Supertype::Host => 6,
            };
            debug_assert!(itm_b <= 0b111);

            b |= ((is_last as u8) << 3) | itm_b;
            b <<= 4;
        }
        write_to.write_all(&[b])?;
    }
    Ok(())
}

impl SerializeMinimal for ColorCombination {
    type ExternalData<'s> = ();

    fn minimally_serialize<'a, 's: 'a, W: std::io::Write>(
        &'a self,
        write_to: &mut W,
        _: Self::ExternalData<'s>,
    ) -> std::io::Result<()> {
        let mut b = bit_sections::Byte::new();
        b.set_bit(0, self.white);
        b.set_bit(1, self.blue);
        b.set_bit(2, self.black);
        b.set_bit(3, self.red);
        b.set_bit(4, self.green);
        b.set_bit(5, self.colorless);

        b.into_inner().minimally_serialize(write_to, ())
    }
}

impl DeserializeFromMinimal for ColorCombination {
    type ExternalData<'d> = ();

    fn deserialize_minimal<'a, 'd: 'a, R: std::io::Read>(
        from: &'a mut R,
        _: Self::ExternalData<'d>,
    ) -> Result<Self, std::io::Error> {
        let bits = bit_sections::Byte::from(u8::deserialize_minimal(from, ())?);

        Ok(ColorCombination {
            white: bits.get_bit(0) == 0b1,
            blue: bits.get_bit(1) == 0b1,
            black: bits.get_bit(2) == 0b1,
            red: bits.get_bit(3) == 0b1,
            green: bits.get_bit(4) == 0b1,
            colorless: bits.get_bit(5) == 0b1,
        })
    }
}

impl DeserializeFromMinimal for ManaCost {
    type ExternalData<'d> = ();

    fn deserialize_minimal<'a, 'd: 'a, R: std::io::Read>(
        from: &'a mut R,
        _: Self::ExternalData<'d>,
    ) -> Result<Self, std::io::Error> {
        Vec::<ManaSymbol>::deserialize_minimal(from, ()).map(Self)
    }
}

impl SerializeMinimal for ManaCost {
    type ExternalData<'s> = ();

    fn minimally_serialize<'a, 's: 'a, W: std::io::Write>(
        &'a self,
        write_to: &mut W,
        _: Self::ExternalData<'s>,
    ) -> std::io::Result<()> {
        self.0.minimally_serialize(write_to, ())
    }
}

impl SerializeMinimal for ManaSymbol {
    type ExternalData<'s> = ();

    fn minimally_serialize<'a, 's: 'a, W: std::io::Write>(
        &'a self,
        write_to: &mut W,
        _: Self::ExternalData<'s>,
    ) -> std::io::Result<()> {
        write_to.write_all(&[mana_symbol_to_byte(self)])
    }
}

impl DeserializeFromMinimal for ManaSymbol {
    type ExternalData<'s> = ();

    fn deserialize_minimal<'a, 'd: 'a, R: std::io::Read>(
        from: &'a mut R,
        _: Self::ExternalData<'d>,
    ) -> Result<Self, std::io::Error> {
        from.read_one().map(byte_to_mana_symbol)
    }
}

fn mana_symbol_to_byte(ms: &ManaSymbol) -> u8 {
    //values 42..=63 in the 6 LSBs with any values for the 2 MSBs
    // are holes in the ConventionalColored representation, and
    // therefore can be used
    // in the other variants without any discriminant.

    match ms {
        ManaSymbol::Snow => 42,
        ManaSymbol::Variable(ManaVariable::X) => 43,
        ManaSymbol::Variable(ManaVariable::Y) => 44,
        ManaSymbol::Variable(ManaVariable::Z) => 45,
        ManaSymbol::GenericNumber(num) => {
            //if wotc makes a card that costs 20 generic mana i am
            //going to kick a can up the road.
            let lsb = (num + 42) as u8;
            debug_assert!(lsb < 63);
            debug_assert!(lsb >= 42);

            0b1100_0000 | (*num as u8)
        }
        ManaSymbol::ConventionalColored {
            phyrexian,
            split_two_generic,
            color,
            split_color,
        } => {
            let mut flags = BitSection::<0, 2, u8>::new();
            flags.set_bit(0, *phyrexian);
            flags.set_bit(1, *split_two_generic);

            let mut symbol_colors: u8 = match split_color {
                Some(Color::White) => 0,
                Some(Color::Blue) => 1,
                Some(Color::Red) => 2,
                Some(Color::Green) => 3,
                Some(Color::Black) => 4,
                Some(Color::Colorless) => 5,
                None => 6,
            };
            symbol_colors *= 6;

            symbol_colors += match color {
                Color::White => 0,
                Color::Blue => 1,
                Color::Red => 2,
                Color::Green => 3,
                Color::Black => 4,
                Color::Colorless => 5,
            };

            debug_assert!(symbol_colors < 42);
            debug_assert!(symbol_colors < 0b1100_0000);

            flags.into_inner() | symbol_colors
        }
    }
}

fn byte_to_mana_symbol(ms: u8) -> ManaSymbol {
    //handle holes that are allocated to other cases
    match ms {
        42 => return ManaSymbol::Snow,
        43 => return ManaSymbol::Variable(ManaVariable::X),
        44 => return ManaSymbol::Variable(ManaVariable::Y),
        45 => return ManaSymbol::Variable(ManaVariable::Z),
        ms if ms >> 6 == 0b11 && ms & 0b0011_1111 >= 42 => {
            return ManaSymbol::GenericNumber((ms & 0b1_1111) as usize);
        }
        _ => {}
    }

    let phyrexian = (ms & 0b1000_0000) > 0;
    let split_two_generic = (ms & 0b0100_0000) > 0;

    let mut color_combo = ms & 0b0011_1111;

    let color = match color_combo % 6 {
        0 => Color::White,
        1 => Color::Blue,
        2 => Color::Red,
        3 => Color::Green,
        4 => Color::Black,
        5 => Color::Colorless,
        _ => unreachable!(),
    };

    color_combo /= 6;

    let split_color = match color_combo {
        0 => Some(Color::White),
        1 => Some(Color::Blue),
        2 => Some(Color::Red),
        3 => Some(Color::Green),
        4 => Some(Color::Black),
        5 => Some(Color::Colorless),
        6 => None,
        //todo: here's where we'd handle any extra cases in the enum
        _ => None,
    };

    ManaSymbol::ConventionalColored {
        phyrexian,
        split_two_generic,
        color,
        split_color,
    }
}
