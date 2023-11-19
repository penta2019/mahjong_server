use serde::{de, ser};

use super::*;
use crate::util::common::{tile_number_from_char, tile_type_from_char};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Tile(pub Type, pub Tnum); // (type index, number index)
pub const Z8: Tile = Tile(TZ, UK); // unknown tile

impl Tile {
    pub fn from_symbol(s: &str) -> Self {
        let cs: Vec<char> = s.chars().collect();
        let t = tile_type_from_char(cs[0]).unwrap();
        let n = tile_number_from_char(cs[1]).unwrap();
        Self(t, n)
    }

    // 赤5の場合,通常の5を返却. それ以外の場合はコピーをそのまま返却.
    #[inline]
    pub fn to_normal(self) -> Self {
        if self.1 == 0 {
            Self(self.0, 5)
        } else {
            self
        }
    }

    // 数牌
    #[inline]
    pub fn is_suit(&self) -> bool {
        self.0 != TZ
    }

    // 字牌
    #[inline]
    pub fn is_hornor(&self) -> bool {
        self.0 == TZ
    }

    // 1,9牌
    #[inline]
    pub fn is_terminal(&self) -> bool {
        self.0 != TZ && (self.1 == 1 || self.1 == 9)
    }

    // 么九牌
    #[inline]
    pub fn is_end(&self) -> bool {
        self.0 == TZ || self.1 == 1 || self.1 == 9
    }

    // 中張牌
    #[inline]
    pub fn is_simple(&self) -> bool {
        !self.is_end()
    }

    // 風牌
    #[inline]
    pub fn is_wind(&self) -> bool {
        self.0 == TZ && self.1 <= WN
    }

    // 三元牌
    #[inline]
    pub fn is_doragon(&self) -> bool {
        self.0 == TZ && DW <= self.1 && self.1 <= DR
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", ['m', 'p', 's', 'z'][self.0], self.1)
    }
}

impl fmt::Debug for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl PartialOrd for Tile {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Tile {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.0 != other.0 {
            return self.0.cmp(&other.0);
        }

        // 赤5は4.5に変換して比較
        let a = if self.1 == 0 { 4.5 } else { self.1 as f32 };
        let b = if other.1 == 0 { 4.5 } else { other.1 as f32 };
        a.partial_cmp(&b).unwrap()
    }
}

impl ser::Serialize for Tile {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

struct TileVisitor;

impl<'de> de::Visitor<'de> for TileVisitor {
    type Value = Tile;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("tile symbol")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Tile::from_symbol(v))
    }
}

impl<'de> de::Deserialize<'de> for Tile {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as de::Deserializer<'de>>::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_identifier(TileVisitor)
    }
}

// [TileTable]
pub type TileRow = [usize; TNUM];
pub type TileTable = [TileRow; TYPE];

// [TileState]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(tag = "t", content = "c")]
pub enum TileState {
    H(Seat),        // Hand
    M(Seat, Index), // Meld
    K(Seat, Index), // Nukidora
    D(Seat, Index), // Discard
    R,              // doRa
    #[default]
    U, // Unknown
}

use TileState::*;

impl fmt::Display for TileState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            H(s) => write!(f, "H{}", s),
            M(s, _) => write!(f, "M{}", s),
            K(s, _) => write!(f, "K{}", s),
            D(s, _) => write!(f, "D{}", s),
            R => write!(f, "R "),
            U => write!(f, "U "),
        }
    }
}
