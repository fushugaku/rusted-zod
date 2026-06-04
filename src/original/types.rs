use bevy::prelude::Color;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum PlanetType {
    Desert = 0,
    Volcanic = 1,
    Arctic = 2,
    Jungle = 3,
    City = 4,
}

impl TryFrom<u8> for PlanetType {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Desert),
            1 => Ok(Self::Volcanic),
            2 => Ok(Self::Arctic),
            3 => Ok(Self::Jungle),
            4 => Ok(Self::City),
            other => Err(format!("unknown planet type {other}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i8)]
pub enum TeamType {
    Null = 0,
    Red = 1,
    Blue = 2,
    Green = 3,
    Yellow = 4,
    Purple = 5,
    Teal = 6,
    White = 7,
    Black = 8,
}

impl TeamType {
    pub fn atlas_team(self) -> Self {
        match self {
            Self::Red | Self::Blue | Self::Green | Self::Yellow => self,
            _ => Self::Red,
        }
    }

    pub fn asset_name(self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Red => "red",
            Self::Blue => "blue",
            Self::Green => "green",
            Self::Yellow => "yellow",
            Self::Purple => "purple",
            Self::Teal => "teal",
            Self::White => "white",
            Self::Black => "black",
        }
    }

    pub fn color(self) -> Color {
        match self {
            Self::Null => Color::srgb(0.5, 0.5, 0.5),
            Self::Red => Color::srgb(0.95, 0.05, 0.04),
            Self::Blue => Color::srgb(0.05, 0.18, 0.95),
            Self::Green => Color::srgb(0.08, 0.75, 0.08),
            Self::Yellow => Color::srgb(0.95, 0.78, 0.18),
            Self::Purple => Color::srgb(0.55, 0.15, 0.75),
            Self::Teal => Color::srgb(0.0, 0.65, 0.65),
            Self::White => Color::srgb(0.92, 0.92, 0.92),
            Self::Black => Color::srgb(0.04, 0.04, 0.04),
        }
    }
}

impl TryFrom<i8> for TeamType {
    type Error = String;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Null),
            1 => Ok(Self::Red),
            2 => Ok(Self::Blue),
            3 => Ok(Self::Green),
            4 => Ok(Self::Yellow),
            5 => Ok(Self::Purple),
            6 => Ok(Self::Teal),
            7 => Ok(Self::White),
            8 => Ok(Self::Black),
            other => Err(format!("unknown team type {other}")),
        }
    }
}
