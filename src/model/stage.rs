use super::*;
use crate::{control::common::tiles_from_tile_table, util::misc::vec_to_string};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Rule {
    pub round: usize,            // ゲーム設定 (1: 4人東, 2: 4人南)
    pub is_sanma: bool,          // 三麻フラグ(未実装, 常にfalse)
    pub initial_score: Score,    // 初期スコア (4麻:25000)
    pub settlement_score: Score, // ゲームが終了して1位が確定するのに必要なスコア (4麻:30000)
    pub red5: usize,             // 赤ドラの枚数 (0~4) 一般的なルールでは1
    pub bust: bool,              // 飛びのアリなし
}

#[derive(Debug, Default, Serialize)]
pub struct Stage {
    pub rule: Rule,                                  // ゲーム設定
    pub round: usize,                                // 場 (東:0, 南:1, 西:2, 北:3)
    pub dealer: usize,                               // 局 (0~3 = 親のseat)
    pub honba_sticks: usize,                         // 本場
    pub riichi_sticks: usize,                        // リーチ棒の供託
    pub turn: Seat,                                  // ツモ番のプレイヤーの座席
    pub step: usize,                                 // ステップ op関数を呼び出す毎に+1する
    pub wall_count: usize,                           // 牌山の残り枚数
    pub doras: Vec<Tile>,                            // ドラ表示牌
    pub discards: Vec<(Seat, Index)>,                // プレイヤー全員の捨て牌
    pub last_tile: Option<(Seat, ActionType, Tile)>, // 他家に鳴き又はロンされる可能性のある牌(捨て牌,加槓,暗槓)
    pub last_riichi: Option<Seat>,                   // リーチがロンされずに成立した場合の供託更新用
    pub players: [Player; SEAT],                     // 各プレイヤー情報
    pub tile_states: [[[TileState; TILE]; TNUM]; TYPE],
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

#[derive(Debug, Default, Serialize)]
pub struct Player {
    pub seat: Seat,               // 座席番号(場・局が変わってもゲーム終了まで不変)
    pub score: Score,             // 得点
    pub hand: TileTable,          // 手牌(4x10の配列)
    pub drawn: Option<Tile>,      // ツモ牌
    pub melds: Vec<Meld>,         // 鳴き一覧
    pub nukidoras: Vec<Nukidora>, // 北抜き
    pub riichi: Option<Index>,    // リーチ宣言牌のdiscardsにおけるindex
    pub discards: Vec<Discard>,   // 捨て牌一覧
    pub pao: Option<Seat>,        // 責任払い, 役満を確定する副露を許したプレイヤーの座席
    pub is_shown: bool,           // 手牌が見えるかどうか 見えない場合,手牌はすべてz8(=unknown)
    pub rank: usize,              // 現在の順位

    // 聴牌 (is_shown = trueの時のみ有効)
    pub winning_tiles: Vec<Tile>, // 聴牌時の和了牌
    pub is_furiten: bool,         // 自分の捨て牌によるフリテン
    pub is_furiten_other: bool,   // 他家の捨て牌の見逃しによるフリテン

    // 条件役用のフラグ 天和,地和,海底など和了のタイミングで発生する役はここに含まない
    pub is_menzen: bool,        // 門前ツモ
    pub is_riichi: bool,        // リーチ (ダブルリーチを含む)
    pub is_daburii: bool,       // ダブルリーチ
    pub is_ippatsu: bool,       // 一発 立直後にセットして次の打牌または他家の鳴きでfalseをセット
    pub is_rinshan: bool,       // 槓の操作中にtrueをセット 打牌でfalseをセット
    pub is_nagashimangan: bool, // 初期値としてtrueをセット, 中張牌や他家に鳴かれたらfalseをセット
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let drawn = if let Some(d) = self.drawn {
            d.to_string()
        } else {
            "None".to_string()
        };
        let hand = vec_to_string(&tiles_from_tile_table(&self.hand));
        let discards = vec_to_string(&self.discards);
        let melds = vec_to_string(&self.melds);
        writeln!(
            f,
            "seat: {}, score: {}, riichi: {:?}, nukidora: {}, drawn: {}",
            self.seat,
            self.score,
            self.riichi,
            self.nukidoras.len(),
            drawn,
        )?;
        writeln!(
            f,
            "furiten: {}, furiten_other: {}, rinshan: {}, winning_tiles: {:?}",
            self.is_furiten, self.is_furiten_other, self.is_rinshan, self.winning_tiles,
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
    pub is_drawn: bool,              // ツモ切りフラグ
    pub meld: Option<(Seat, Index)>, // 鳴きが入った場合にセット
}

impl fmt::Display for Discard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tile)
    }
}

#[derive(Debug, Serialize)]
pub struct Nukidora {
    pub step: usize,
    pub is_drawn: bool, // ツモってきた牌かどうか
}

impl fmt::Display for Nukidora {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meld {
    pub step: usize,
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
