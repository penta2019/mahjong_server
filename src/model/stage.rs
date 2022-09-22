use super::*;
use crate::util::common::vec_to_string;

#[derive(Debug, Default, Serialize)]
pub struct Stage {
    pub round: usize,                                // 場 (東:0, 南:1, 西:2, 北:3)
    pub dealer: usize,                               // 局 (0~3 = 親のseat)
    pub honba_sticks: usize,                         // 本場
    pub riichi_sticks: usize,                        // リーチ棒の供託
    pub turn: Seat,                                  // ツモ番のプレイヤーの座席
    pub step: usize,                                 // ステップ op関数を呼び出す毎に+1する
    pub wall_count: usize,                           // 牌山の残り枚数
    pub doras: Vec<Tile>,                            // ドラ表示牌
    pub discards: Vec<(Seat, Index)>,                // プレイヤー全員の捨て牌
    pub last_tile: Option<(Seat, ActionType, Tile)>, // 他家にロンされる可能性のある牌(捨て牌,槍槓) フリテン判定用
    pub last_riichi: Option<Seat>,                   // リーチがロンされずに成立した場合の供託更新用
    pub players: [Player; SEAT],                     // 各プレイヤー情報
    pub is_3p: bool,                                 // 三麻フラグ(未実装, 常にfalse)
    pub tile_states: [[[TileState; TILE]; TNUM]; TYPE],
}

impl Stage {
    #[inline]
    pub fn is_dealer(&self, seat: Seat) -> bool {
        seat == self.dealer
    }

    #[inline]
    pub fn get_prevalent_wind(&self) -> Tnum {
        self.round % SEAT + 1 // WE | WS | WW | WN
    }

    #[inline]
    pub fn get_seat_wind(&self, seat: Seat) -> Tnum {
        (seat + SEAT - self.dealer) % SEAT + 1 // WE | WS | WW | WN
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "round: {}, dealer: {}, honba_sticks: {}, riichi_sticks: {}",
            self.round, self.dealer, self.honba_sticks, self.riichi_sticks,
        )?;
        writeln!(
            f,
            "turn: {}, wall_count: {}, doras: {}, last_tile: {:?}",
            self.turn,
            self.wall_count,
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
            for ni in 1..TNUM {
                write!(f, "{}{} ", ['m', 'p', 's', 'z'][ti], ni)?;
            }
            writeln!(f)?;
            write!(f, "--------------------------")?;
            for pi in 0..TILE {
                writeln!(f)?;
                for ni in 1..TNUM {
                    write!(f, "{} ", self.tile_states[ti][ni][pi])?;
                }
            }
        }
        writeln!(f)?;

        Ok(())
    }
}
