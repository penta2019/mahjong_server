use super::*;
use crate::util::common::vec_to_string;

use TileStateType::*;

pub const Z8: Tile = Tile(TZ, UK); // unknown tile
pub type TileRow = [usize; TNUM];
pub type TileTable = [TileRow; TYPE];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(tag = "t", content = "c")]
pub enum TileStateType {
    H(Seat),        // Hand
    M(Seat, Index), // Meld
    K(Seat, Index), // Kita
    D(Seat, Index), // Discard
    R,              // doRa
    U,              // Unknown
}

impl Default for TileStateType {
    fn default() -> Self {
        U
    }
}

impl fmt::Display for TileStateType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

#[derive(Debug, Default, Serialize)]
pub struct Stage {
    pub bakaze: usize,                               // 場 (東:0, 南:1, 西:2, 北:3)
    pub kyoku: usize,                                // 局 (0~3 = 親のseat)
    pub honba: usize,                                // 本場
    pub kyoutaku: usize,                             // リーチ棒の供託
    pub turn: Seat,                                  // ツモ番のプレイヤーの座席
    pub step: usize,                                 // ステップ op関数を呼び出す毎に+1する
    pub left_tile_count: usize,                      // 牌山の残り枚数
    pub doras: Vec<Tile>,                            // ドラ表示牌
    pub discards: Vec<(Seat, Index)>,                // プレイヤー全員の捨て牌
    pub last_tile: Option<(Seat, ActionType, Tile)>, // 他家にロンされる可能性のある牌(捨て牌,槍槓) フリテン判定用
    pub last_riichi: Option<Seat>,                   // リーチがロンされずに成立した場合の供託更新用
    pub players: [Player; SEAT],                     // 各プレイヤー情報
    pub is_3p: bool,                                 // 三麻フラグ(未実装, 常にfalse)
    pub tile_states: [[[TileStateType; TILE]; TNUM]; TYPE],
}

impl Stage {
    #[inline]
    pub fn is_leader(&self, seat: Seat) -> bool {
        seat == self.kyoku
    }

    #[inline]
    pub fn get_prevalent_wind(&self) -> Tnum {
        self.bakaze % SEAT + 1 // WE | WS | WW | WN
    }

    #[inline]
    pub fn get_seat_wind(&self, seat: Seat) -> Tnum {
        (seat + SEAT - self.kyoku) % SEAT + 1 // WE | WS | WW | WN
    }

    pub fn get_scores(&self) -> [Score; SEAT] {
        let mut scores = [0; SEAT];
        for s in 0..SEAT {
            scores[s] = self.players[s].score;
        }
        scores
    }
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "bakaze: {}, kyoku: {}, honba: {}, kyoutaku: {}",
            self.bakaze, self.kyoku, self.honba, self.kyoutaku,
        )?;
        writeln!(
            f,
            "turn: {}, left_tile_count: {}, doras: {}, last_tile: {:?}",
            self.turn,
            self.left_tile_count,
            vec_to_string(&self.doras),
            self.last_tile,
        )?;
        writeln!(f)?;

        let boader = "-".to_string().repeat(80);
        write!(f, "{}", boader)?;
        for p in &self.players {
            writeln!(f)?;
            writeln!(f, "{}", p)?;
            write!(f, "{}", boader)?;
        }

        for ti in 0..TYPE {
            writeln!(f)?;
            writeln!(f)?;
            for i in 1..TNUM {
                write!(f, "{}{} ", ['m', 'p', 's', 'z'][ti], i)?;
            }
            writeln!(f)?;
            write!(f, "--------------------------")?;
            for pi in 0..TILE {
                writeln!(f)?;
                for i in 1..TNUM {
                    write!(f, "{} ", self.tile_states[ti][i][pi])?;
                }
            }
        }
        writeln!(f)?;

        Ok(())
    }
}

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
