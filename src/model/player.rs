use super::*;
use crate::util::common::vec_to_string;

#[derive(Debug, Default, Serialize)]
pub struct Player {
    pub seat: Seat,             // 座席番号(場・局が変わってもゲーム終了まで不変)
    pub score: Score,           // 得点
    pub hand: TileTable,        // 手牌(4x10の配列)
    pub drawn: Option<Tile>,    // ツモ牌
    pub melds: Vec<Meld>,       // 鳴き一覧
    pub kitas: Vec<Kita>,       // 北抜き vecの中身はすべてTile(TZ, TN)
    pub riichi: Option<Index>,  // リーチ宣言牌のdiscardsにおけるindex
    pub discards: Vec<Discard>, // 捨て牌一覧
    pub is_shown: bool,         // 手牌が見えるかどうか 見えない場合,手牌はすべてz8(=unknown)
    pub rank: usize,            // 現在の順位

    // 聴牌
    pub win_tiles: Vec<Tile>,   // 聴牌時の和了牌
    pub is_furiten: bool,       // 自分の捨て牌によるフリテン
    pub is_furiten_other: bool, // 他家の捨て牌の見逃しによるフリテン

    // 条件役用のフラグ 天和,地和,海底など和了のタイミングで発生する役はここに含まない
    pub is_menzen: bool,  // 門前ツモ
    pub is_riichi: bool,  // リーチ (ダブルリーチを含む)
    pub is_daburii: bool, // ダブルリーチ
    pub is_ippatsu: bool, // 一発 立直後にセットして次の打牌または他家の鳴きでfalseをセット
    pub is_rinshan: bool, // 槓の操作中にtrueをセット 打牌でfalseをセット
}

impl Player {
    pub fn count_tile(&self, t: Tile) -> usize {
        let h = &self.hand;
        if t.1 == 5 {
            h[t.0][t.1] - h[t.0][0]
        } else {
            h[t.0][t.1]
        }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut hand = vec![];
        for ti in 0..TYPE {
            for ni in 1..TNUM {
                let e = self.hand[ti][ni];
                for c in 0..e {
                    if ti != TZ && ni == 5 && c == 0 && self.hand[ti][0] == 1 {
                        hand.push(Tile(ti, 0)); // 赤5
                    } else {
                        hand.push(Tile(ti, ni));
                    }
                }
            }
        }
        let drawn = if let Some(d) = self.drawn {
            d.to_string()
        } else {
            "None".to_string()
        };
        let hand = vec_to_string(&hand);
        let discards = vec_to_string(&self.discards);
        let melds = vec_to_string(&self.melds);
        writeln!(
            f,
            "seat: {}, score: {}, riichi: {:?}, kita: {}, drawn: {}",
            self.seat,
            self.score,
            self.riichi,
            self.kitas.len(),
            drawn,
        )?;
        writeln!(
            f,
            "furiten: {}, furiten_other: {}, rinshan: {}, win_tiles: {:?}",
            self.is_furiten, self.is_furiten_other, self.is_rinshan, self.win_tiles,
        )?;
        writeln!(f, "hand: {}", hand)?;
        writeln!(f, "melds: {}", melds)?;
        write!(f, "discards: {}", discards)
    }
}

#[derive(Debug, Serialize)]
pub struct Discard {
    pub step: usize,
    pub tile: Tile,
    pub drawn: bool,                 // ツモ切りフラグ
    pub meld: Option<(Seat, Index)>, // 鳴きが入った場合にセット
}

impl fmt::Display for Discard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tile)
    }
}

#[derive(Debug, Serialize)]
pub struct Kita {
    pub step: usize,
    pub seat: Seat,
    pub drawn: bool,
}

impl fmt::Display for Kita {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "z4")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeldType {
    Chi,
    Pon,
    Minkan,
    Kakan,
    Ankan,
}

#[derive(Debug, Serialize)]
pub struct Meld {
    pub step: usize,
    pub seat: Seat,
    pub meld_type: MeldType,
    pub tiles: Vec<Tile>,
    pub froms: Vec<Seat>,
}

impl fmt::Display for Meld {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let z = self.tiles.iter().zip(self.froms.iter());
        let s: Vec<String> = z.map(|x| format!("{}({})", x.0, x.1)).collect();
        write!(f, "{}", s.join("|"))
    }
}
