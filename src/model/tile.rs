use serde::{de, ser};

use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Tile(pub Type, pub Tnum); // (type index, number index)
pub const Z8: Tile = Tile(TZ, UK); // unknown tile

impl Tile {
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

    fn from_symbol(s: &str) -> Self {
        let b = s.as_bytes();
        let n = b[1] - b'0';
        let t = match b[0] as char {
            'm' => 0,
            'p' => 1,
            's' => 2,
            'z' => 3,
            _ => panic!("invalid Tile type"),
        };
        Self(t, n as usize)
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
        if self.0 != other.0 {
            return Some(self.0.cmp(&other.0));
        }

        // 赤5は4.5に変換して比較
        let a = if self.1 == 0 { 4.5 } else { self.1 as f32 };
        let b = if other.1 == 0 { 4.5 } else { other.1 as f32 };
        a.partial_cmp(&b)
    }
}

impl Ord for Tile {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
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

pub fn tiles_from_tile_table(tt: &TileTable) -> Vec<Tile> {
    let mut hand = vec![];
    for ti in 0..TYPE {
        for ni in 1..TNUM {
            // 赤5
            if ni == 5 {
                for _ in 0..tt[ti][0] {
                    hand.push(Tile(ti, 0));
                }
            }

            for _ in 0..tt[ti][ni] {
                hand.push(Tile(ti, ni));
            }
        }
    }
    hand
}

pub fn tiles_with_red5(tt: &TileTable, t: Tile) -> Vec<Tile> {
    if tt[t.0][t.1] == 0 {
        return vec![];
    }

    let Tile(ti, ni) = t;
    let tr = tt[ti];
    if ni != 5 {
        return vec![t]; // 5ではない場合
    }
    if tr[0] == 0 {
        return vec![t]; // 通常5しかない場合
    }
    if tr[0] == tr[5] {
        return vec![Tile(ti, 0)]; // 赤5しかない場合
    }
    vec![t, Tile(ti, 0)] // 通常5と赤5の療法がある場合
}

// [TileState]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(tag = "t", content = "c")]
pub enum TileState {
    H(Seat),        // Hand
    M(Seat, Index), // Meld
    K(Seat, Index), // Kita
    D(Seat, Index), // Discard
    R,              // doRa
    U,              // Unknown
}

use TileState::*;

impl Default for TileState {
    fn default() -> Self {
        U
    }
}

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
