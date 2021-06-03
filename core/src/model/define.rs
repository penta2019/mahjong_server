// 型エイリアス
pub type Seat = usize; // 座席
pub type Type = usize; // 牌の種別部分 (萬子,筒子,索子,字牌)
pub type Tnum = usize; // 牌の数字部分 (1~9, 0:赤5 の10種)
pub type Index = usize; // その他Index

// Number
pub const SEAT: usize = 4; // 座席の数
pub const TYPE: usize = 4; // 牌の種別部分の数 (萬子,筒子,索子,字牌)
pub const TNUM: usize = 10; // 牌の数字部分の数 (1~9, 0:赤5 の10種)
pub const TILE: usize = 4; // 同種の牌の数

// Type Index
pub const TM: Type = 0; // Type: Manzu (萬子)
pub const TP: Type = 1; // Type: Pinzu (筒子)
pub const TS: Type = 2; // Type: Souzu (索子)
pub const TZ: Type = 3; // Type: Zihai (字牌)

// Tnum Index
pub const WE: Tnum = 1; // Wind:    East  (東)
pub const WS: Tnum = 2; // Wind:    South (南)
pub const WW: Tnum = 3; // Wind:    West  (西)
pub const WN: Tnum = 4; // Wind:    North (北)
pub const DW: Tnum = 5; // Doragon: White (白)
pub const DG: Tnum = 6; // Doragon: Green (發)
pub const DR: Tnum = 7; // Doragon: Red   (中)
pub const UK: Tnum = 8; // Unknown

pub const NO_SEAT: Seat = 4;
