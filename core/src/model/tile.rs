use super::*;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Tile(pub Type, pub Tnum); // (type index, number index)

impl Tile {
    // number index(赤5考慮)を返却
    #[inline]
    pub fn n(&self) -> Tnum {
        if self.1 == 0 {
            5
        } else {
            self.1
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", ['m', 'p', 's', 'z'][self.0], self.1)
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
        self.partial_cmp(&other).unwrap()
    }
}
