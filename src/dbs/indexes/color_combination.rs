use minimal_storage::{
    serialize_fast::MinimalSerdeFast,
    serialize_min::{DeserializeFromMinimal, SerializeMinimal},
};
use ratatui::layout::Direction;
use tree::tree_traits::{Dimension, MinValue, MultidimensionalKey, MultidimensionalParent, Zero};

use crate::data_model::card::{Color, ColorCombination};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct ColorCombinationMaybe {
    pub white: Option<bool>,
    pub blue: Option<bool>,
    pub black: Option<bool>,
    pub red: Option<bool>,
    pub green: Option<bool>,
    pub colorless: Option<bool>,
}

impl SerializeMinimal for ColorCombinationMaybe {
    type ExternalData<'s> = ();

    fn minimally_serialize<'a, 's: 'a, W: std::io::Write>(
        &'a self,
        write_to: &mut W,
        _external_data: Self::ExternalData<'s>,
    ) -> std::io::Result<()> {
        let mut num = 0usize;

        num += self.white.map(Into::into).unwrap_or(2usize);
        num *= 3;
        num += self.blue.map(Into::into).unwrap_or(2usize);
        num *= 3;
        num += self.black.map(Into::into).unwrap_or(2usize);
        num *= 3;
        num += self.red.map(Into::into).unwrap_or(2usize);
        num *= 3;
        num += self.green.map(Into::into).unwrap_or(2usize);
        num *= 3;
        num += self.colorless.map(Into::into).unwrap_or(2usize);

        num.minimally_serialize(write_to, ())
    }
}

impl DeserializeFromMinimal for ColorCombinationMaybe {
    type ExternalData<'d> = ();

    fn deserialize_minimal<'a, 'd: 'a, R: std::io::Read>(
        from: &'a mut R,
        _external_data: Self::ExternalData<'d>,
    ) -> Result<Self, std::io::Error> {
        let mut num = usize::deserialize_minimal(from, ())?;

        let colorless = option_bool_from_idx(num);
        num /= 3;
        let green = option_bool_from_idx(num);
        num /= 3;
        let red = option_bool_from_idx(num);
        num /= 3;
        let black = option_bool_from_idx(num);
        num /= 3;
        let blue = option_bool_from_idx(num);
        num /= 3;
        let white = option_bool_from_idx(num);

        Ok(Self {
            white,
            blue,
            black,
            red,
            green,
            colorless,
        })
    }
}

fn option_bool_from_idx(i: usize) -> Option<bool> {
    match i % 3 {
        0 => Some(false),
        1 => Some(true),
        2 => None,
        _ => unreachable!(),
    }
}

impl Dimension<6> for Color {
    fn next_axis(&self) -> Self {
        match self {
            Color::White => Color::Blue,
            Color::Blue => Color::Red,
            Color::Red => Color::Green,
            Color::Green => Color::Black,
            Color::Black => Color::Colorless,
            Color::Colorless => Color::White,
        }
    }

    fn from_index(index: usize) -> Self {
        match index % 6 {
            0 => Color::White,
            1 => Color::Blue,
            2 => Color::Red,
            3 => Color::Green,
            4 => Color::Black,
            5 => Color::Colorless,
            _ => unreachable!(),
        }
    }

    fn arbitrary_first() -> Self {
        Color::White
    }
}

impl MultidimensionalParent<6> for ColorCombinationMaybe {
    type DimensionEnum = Color;

    const UNIVERSE: Self = ColorCombinationMaybe {
        white: None,
        blue: None,
        black: None,
        red: None,
        green: None,
        colorless: None,
    };

    fn contains(&self, child: &Self) -> bool {
        (self.white.is_none() || self.white == child.white)
            && (self.blue.is_none() || self.blue == child.blue)
            && (self.red.is_none() || self.red == child.red)
            && (self.green.is_none() || self.green == child.green)
            && (self.black.is_none() || self.black == child.black)
            && (self.colorless.is_none() || self.colorless == child.colorless)
    }

    fn overlaps(&self, child: &Self) -> bool {
        (self.white.is_none() || child.white.is_none() || self.white == child.white)
            || (self.blue.is_none() || child.blue.is_none() || self.blue == child.blue)
            || (self.red.is_none() || child.red.is_none() || self.red == child.red)
            || (self.green.is_none() || child.green.is_none() || self.green == child.green)
            || (self.black.is_none() || child.black.is_none() || self.black == child.black)
            || (self.colorless.is_none()
                || child.colorless.is_none()
                || self.colorless == child.colorless)
    }

    fn split_evenly_on_dimension(&self, dimension: &Self::DimensionEnum) -> (Self, Self) {
        let mut left = self.to_owned();
        let mut right = self.to_owned();

        match dimension {
            Color::White => {
                left.white.get_or_insert(false);
                right.white.get_or_insert(true);
            }
            Color::Blue => {
                left.blue.get_or_insert(false);
                right.blue.get_or_insert(true);
            }
            Color::Red => {
                left.red.get_or_insert(false);
                right.red.get_or_insert(true);
            }
            Color::Green => {
                left.green.get_or_insert(false);
                right.green.get_or_insert(true);
            }
            Color::Black => {
                left.black.get_or_insert(false);
                right.black.get_or_insert(true);
            }
            Color::Colorless => {
                left.colorless.get_or_insert(false);
                right.colorless.get_or_insert(true);
            }
        }

        (left, right)
    }
}

impl MultidimensionalKey<6> for ColorCombination {
    type Parent = ColorCombinationMaybe;

    type DeltaFromParent = Self;

    type DeltaFromSelfAsChild = Self;

    fn is_contained_in(&self, parent: &Self::Parent) -> bool {
        (parent.white.is_none() || Some(self.white) == parent.white)
            && (parent.blue.is_none() || Some(self.blue) == parent.blue)
            && (parent.red.is_none() || Some(self.red) == parent.red)
            && (parent.green.is_none() || Some(self.green) == parent.green)
            && (parent.black.is_none() || Some(self.black) == parent.black)
            && (parent.colorless.is_none() || Some(self.colorless) == parent.colorless)
    }

    fn delta_from_parent(&self, parent: &Self::Parent) -> Self::DeltaFromParent {
        self.to_owned()
    }

    fn apply_delta_from_parent(delta: &Self::DeltaFromParent, parent: &Self::Parent) -> Self {
        delta.to_owned()
    }

    fn smallest_key_in(parent: &Self::Parent) -> Self {
        Self {
            white: parent.white.unwrap_or(false),
            blue: parent.blue.unwrap_or(false),
            black: parent.black.unwrap_or(false),
            red: parent.red.unwrap_or(false),
            green: parent.green.unwrap_or(false),
            colorless: parent.colorless.unwrap_or(false),
        }
    }

    fn largest_key_in(parent: &Self::Parent) -> Self {
        Self {
            white: parent.white.unwrap_or(true),
            blue: parent.blue.unwrap_or(true),
            black: parent.black.unwrap_or(true),
            red: parent.red.unwrap_or(true),
            green: parent.green.unwrap_or(true),
            colorless: parent.colorless.unwrap_or(true),
        }
    }

    fn delta_from_self(
        finl: &Self::DeltaFromParent,
        initil: &Self::DeltaFromParent,
    ) -> Self::DeltaFromSelfAsChild {
        finl.to_owned()
    }

    fn apply_delta_from_self(
        delta: &Self::DeltaFromSelfAsChild,
        initial: &Self::DeltaFromParent,
    ) -> Self::DeltaFromParent {
        delta.to_owned()
    }
}

impl MinimalSerdeFast for ColorCombination {
    fn fast_minimally_serialize<'a, 's: 'a, W: std::io::Write>(
        &'a self,
        write_to: &mut W,
        _external_data: <Self as SerializeMinimal>::ExternalData<'s>,
    ) -> std::io::Result<()> {
        self.minimally_serialize(write_to, ())
    }

    fn fast_deserialize_minimal<'a, 'd: 'a, R: std::io::Read>(
        from: &'a mut R,
        _external_data: <Self as DeserializeFromMinimal>::ExternalData<'d>,
    ) -> Result<Self, std::io::Error> {
        Self::deserialize_minimal(from, ())
    }

    fn fast_seek_after<R: std::io::Read>(from: &mut R) -> std::io::Result<()> {
        //reads exactly 1 byte to skip past this one
        from.read_exact(&mut [0u8])
    }
}

impl MinValue for ColorCombination {
    const MIN: Self = ColorCombination {
        white: false,
        blue: false,
        black: false,
        red: false,
        green: false,
        colorless: false,
    };
}
