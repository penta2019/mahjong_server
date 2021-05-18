use std::fmt;

use serde::Serialize;

use crate::hand::evaluate::WinContext;
use crate::hand::win::*;
use crate::util::common::{rank_by_rank_vec, vec_to_string};

use MeldType::*;
use TileStateType::*;

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
pub const TM: usize = 0; // Type: Manzu (萬子)
pub const TP: usize = 1; // Type: Pinzu (筒子)
pub const TS: usize = 2; // Type: Souzu (索子)
pub const TZ: usize = 3; // Type: Zihai (字牌)

// Tnum Index
pub const WE: usize = 1; // Wind:    East  (東)
pub const WS: usize = 2; // Wind:    South (南)
pub const WW: usize = 3; // Wind:    West  (西)
pub const WN: usize = 4; // Wind:    North (北)
pub const DW: usize = 5; // Doragon: White (白)
pub const DG: usize = 6; // Doragon: Green (發)
pub const DR: usize = 7; // Doragon: Red   (中)
pub const UK: usize = 8; // Unknown

// Tile =======================================================================

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

pub const Z8: Tile = Tile(TZ, UK); // unknown tile

pub type TileRow = [usize; TNUM];
pub type TileTable = [TileRow; TYPE];

// Discard ====================================================================

#[derive(Debug, Serialize)]
pub struct Discard {
    pub step: usize,
    pub tile: Tile,
    pub drawn: bool,                 // ツモ切りフラグ
    pub meld: Option<(Seat, Index)>, // 鳴きが入った場合にセット
}

impl fmt::Display for Discard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.tile.to_string())
    }
}

// Meld =======================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MeldType {
    Chii,
    Pon,
    Minkan,
    Kakan,
    Ankan,
}

#[derive(Debug, Serialize)]
pub struct Meld {
    pub step: usize,
    pub seat: Seat,
    pub type_: MeldType,
    pub tiles: Vec<Tile>,
    pub froms: Vec<Seat>,
}

impl fmt::Display for Meld {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let z = self.tiles.iter().zip(self.froms.iter());
        let s: Vec<String> = z.map(|x| format!("{}({})", x.0, x.1)).collect();
        write!(f, "{}", s.join("|"))
    }
}

// Kita =======================================================================

#[derive(Debug, Serialize)]
pub struct Kita {
    pub step: usize,
    pub seat: Seat,
    pub drawn: bool,
}

impl fmt::Display for Kita {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "z4")
    }
}

// Player =====================================================================

#[derive(Debug, Default, Serialize)]
pub struct Player {
    pub seat: Seat,             // 座席番号(場・局が変わってもゲーム終了まで不変)
    pub score: i32,             // 得点
    pub hand: TileTable,        // 手牌(4x10の配列)
    pub drawn: Option<Tile>,    // ツモ牌
    pub melds: Vec<Meld>,       // 鳴き一覧
    pub kitas: Vec<Kita>,       // 北抜き vecの中身はすべてTile(TZ, TN)
    pub riichi: Option<Index>,  // リーチ宣言牌のdiscardsにおけるindex
    pub discards: Vec<Discard>, // 捨て牌一覧
    pub is_shown: bool,         // 手牌が見えるかどうか 見えない場合、手牌はすべてz8(=unknown)
    pub rank: usize,            // 現在の順位

    // 聴牌
    pub winning_tiles: Vec<Tile>, // 聴牌時の和了牌
    pub is_furiten: bool,         // 自分の捨て牌によるフリテン
    pub is_furiten_other: bool,   // 他家の捨て牌の見逃しによるフリテン

    // 条件役用のフラグ 天和,地和,海底など和了のタイミングで発生する役はここに含まない
    pub is_menzen: bool,  // 門前ツモ
    pub is_riichi: bool,  // リーチ (ダブルリーチを含む)
    pub is_daburii: bool, // ダブルリーチ
    pub is_ippatsu: bool, // 一発 立直後にセットして次の打牌または他家の鳴きでfalseをセット
    pub is_rinshan: bool, // 槓の操作中にtrueをセット 打牌でfalseをセット
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
        write!(
            f,
            "seat: {}, score: {}, riichi: {:?}, kita: {}, drawn: {}\n",
            self.seat,
            self.score,
            self.riichi,
            self.kitas.len(),
            drawn,
        )?;
        write!(
            f,
            "furiten: {}, furiten_other: {}, rinshan: {}, winning_tiles: {:?}\n",
            self.is_furiten, self.is_furiten_other, self.is_rinshan, self.winning_tiles,
        )?;
        write!(
            f,
            "hand:  {}\n\
            melds: {}\n\
            discards:  {}",
            hand, melds, discards
        )
    }
}

// misc =======================================================================

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DrawType {
    Kyushukyuhai,   // 九種九牌
    Suufuurenda,    // 四風連打
    Suukansanra,    // 四槓散了
    Suuchariichi,   // 四家立直
    Kouhaiheikyoku, // 荒廃平局
}

// Stage ======================================================================

#[derive(Debug, Default, Serialize)]
pub struct Stage {
    pub round: usize,                    // 場 (東:0, 南:1, 西:2, 北:3)
    pub hand: usize,                     // 局 (0~3 = 親のseat)
    pub ben: usize,                      // 本場
    pub riichi_sticks: usize,            // リーチ棒の供託
    pub turn: Seat,                      // 順番
    pub step: usize,                     // ステップ op関数を呼び出す毎に+1する
    pub left_tile_count: usize,          // 牌山の残り枚数
    pub doras: Vec<Tile>,                // ドラ表示牌
    pub discards: Vec<(Seat, Index)>,    // プレイヤー全員の捨て牌
    pub last_tile: Option<(Seat, Tile)>, // 他家にロンされる可能性のある牌(捨て牌,槍槓) フリテン判定用
    pub players: [Player; SEAT],         // 各プレイヤー情報
    pub is_3p: bool,                     // 三麻フラグ(未実装, 常にfalse)
    pub tile_states: [[[TileStateType; TILE]; TNUM]; TYPE],
}

impl Stage {
    pub fn new(initila_score: i32) -> Self {
        let mut stg = Self::default();
        for s in 0..SEAT {
            let pl = &mut stg.players[s];
            pl.rank = s;
            pl.score = initila_score;
        }
        stg
    }

    pub fn table_edit(&mut self, tile: Tile, old: TileStateType, new: TileStateType) {
        let te = &mut self.tile_states[tile.0][tile.n()];
        // println!("[table_edit] {}: {:?} | {:?} => {:?}", tile, te, old, new);
        let i = te.iter().position(|&x| x == old).unwrap();
        te[i] = new.clone();
        te.sort();
    }

    pub fn print(&self) {
        println!(
            "round: {}, hand: {}, ben: {}, riichi_sticks: {}\n\
            turn: {}, left_tile_count: {}, doras: {}, last_tile: {:?}",
            self.round,
            self.hand,
            self.ben,
            self.riichi_sticks,
            self.turn,
            self.left_tile_count,
            vec_to_string(&self.doras),
            self.last_tile,
        );
        println!();
        println!("-----------------------------------------------------------");
        for p in &self.players {
            println!("{}", p);
            println!("-----------------------------------------------------------");
        }
        println!();
        for ti in 0..TYPE {
            for i in 1..TNUM {
                print!("{}{} ", ['m', 'p', 's', 'z'][ti], i);
            }
            println!();
            println!("--------------------------");
            for pi in 0..TILE {
                for i in 1..TNUM {
                    print!("{} ", self.tile_states[ti][i][pi]);
                }
                println!();
            }
            println!();
        }
    }

    pub fn add_dora(&mut self, tile: Tile) {
        self.table_edit(tile, TileStateType::U, TileStateType::R);
        self.doras.push(tile);
    }

    pub fn op_roundnew(
        &mut self,
        round: usize,
        hand: usize,
        ben: usize,
        riichi_sticks: usize,
        doras: &Vec<Tile>,
        scores: &[i32; SEAT],
        player_hands: &[Vec<Tile>; SEAT],
    ) {
        std::mem::swap(self, &mut Self::default());

        self.round = round;
        self.hand = hand;
        self.ben = ben;
        self.riichi_sticks = riichi_sticks;
        self.step = 1;
        self.left_tile_count = 69;

        for s in 0..SEAT {
            let ph = &player_hands[s];
            let pl = &mut self.players[s];
            pl.seat = s;
            pl.score = scores[s];
            pl.is_shown = !ph.is_empty();
            pl.is_menzen = true;

            self.turn = s; // player_inc_tile() 用
            if pl.is_shown {
                for &t in ph {
                    self.player_inc_tile(t);
                    self.table_edit(t, U, H(s));
                }
            } else {
                pl.hand[TZ][UK] = if s == hand { 14 } else { 13 }; // 親:14, 子:13
            }
        }
        self.turn = hand;
        self.players[hand].drawn = Some(Z8); // 天和のツモ判定unwrap()対策

        for &d in doras {
            self.table_edit(d, U, R);
        }
        self.doras = doras.clone();
    }

    pub fn op_dealtile(&mut self, seat: Seat, tile: Option<Tile>) {
        self.update_furiten_other();

        let s = seat;

        self.step += 1;
        self.turn = s;
        self.left_tile_count -= 1;

        if self.players[s].is_rinshan {
            // 槍槓リーチ一発を考慮して加槓の成立が確定したタイミングで一発フラグをリセット
            self.disable_ippatsu();
        }

        match tile {
            Some(t) => {
                self.player_inc_tile(t);
                self.table_edit(t, U, H(s));
                self.players[s].drawn = Some(t);
            }
            None => {
                self.player_inc_tile(Z8);
            }
        }
    }

    pub fn op_discardtile(&mut self, seat: Seat, tile: Tile, is_drawn: bool, is_riichi: bool) {
        let s = seat;
        let t = tile;
        let riichi_no_meld = is_riichi && self.players.iter().all(|pl| pl.melds.is_empty());

        self.step += 1;
        self.turn = s;

        let pl = &mut self.players[s];
        pl.is_rinshan = false;
        pl.is_furiten_other = false;
        let is_drawn = if pl.is_shown {
            if pl.drawn == Some(t) {
                if pl.hand[t.0][t.1] == 1 {
                    true
                } else {
                    is_drawn
                }
            } else {
                false
            }
        } else {
            is_drawn
        };
        if is_riichi {
            assert!(pl.riichi == None);
            pl.riichi = Some(pl.discards.len());
            pl.is_riichi = true;
            pl.is_ippatsu = true;
            if pl.discards.is_empty() && riichi_no_meld {
                pl.is_daburii = true;
            }
            pl.score -= 1000;
            self.riichi_sticks += 1;
        } else {
            pl.is_ippatsu = false;
        }

        let idx = pl.discards.len();
        let d = Discard {
            step: self.step,
            tile: t,
            drawn: is_drawn,
            meld: None,
        };

        if pl.is_shown {
            self.player_dec_tile(t);
            self.table_edit(t, H(s), D(s, idx));
        } else {
            self.player_dec_tile(Z8);
            self.table_edit(t, U, D(s, idx));
        }

        self.discards.push((s, idx));
        self.players[s].discards.push(d);

        // 和了牌とフリテンの計算
        let pl = &mut self.players[s];
        if pl.is_shown {
            if pl.drawn != Some(t) {
                pl.is_furiten = false;

                // 和了牌を集計
                let mut wait = vec![];
                wait.append(&mut calc_tiles_to_kokushimusou_win(&pl.hand));
                wait.append(&mut calc_tiles_to_normal_win(&pl.hand));
                wait.append(&mut calc_tiles_to_chiitoitsu_win(&pl.hand));
                wait.sort();
                let mut dedup = vec![];
                if !wait.is_empty() {
                    // フリテンの確認
                    let mut tt = TileTable::default();
                    for w in wait {
                        if tt[w.0][w.1] == 0 {
                            tt[w.0][w.1] = 1;
                            dedup.push(w);
                        }
                    }
                    for d in &pl.discards {
                        if tt[d.tile.0][d.tile.n()] != 0 {
                            pl.is_furiten = true;
                            break;
                        }
                    }
                }
                pl.winning_tiles = dedup;
            } else if pl.winning_tiles.contains(&t) {
                pl.is_furiten = true;
            }
        }
        pl.drawn = None;

        self.last_tile = Some((s, t));
    }

    pub fn op_chiiponkan(
        &mut self,
        seat: Seat,
        meld_type: MeldType,
        tiles: &Vec<Tile>,
        froms: &Vec<Seat>,
    ) {
        self.update_furiten_other();

        let s = seat;

        self.step += 1;
        self.turn = s;
        self.disable_ippatsu();

        let pl = &mut self.players[s];
        pl.is_menzen = false;
        if meld_type == MeldType::Minkan {
            pl.is_rinshan = true;
        }

        let idx = pl.melds.len();
        let m = Meld {
            step: self.step,
            seat: s,
            type_: meld_type,
            tiles: tiles.clone(),
            froms: froms.clone(),
        };
        let &(prev_s, prev_i) = self.discards.last().unwrap();
        self.players[prev_s].discards[prev_i].meld = Some((s, idx));

        let mut tf: Vec<(&Tile, &Seat)> = m.tiles.iter().zip(m.froms.iter()).collect();
        tf.sort();
        for (&t, &f) in tf {
            if f == s {
                let pl = &mut self.players[s];
                if pl.is_shown {
                    self.player_dec_tile(t);
                    self.table_edit(t, H(s), M(s, idx));
                } else {
                    self.player_dec_tile(Z8);
                    self.table_edit(t, U, M(s, idx));
                }
            }
        }

        self.players[s].melds.push(m);
    }

    pub fn op_ankankakan(&mut self, seat: Seat, meld_type: MeldType, tile: Tile) {
        self.update_furiten_other();

        let s = seat;

        self.step += 1;
        // self.disable_ippatsu(); 槍槓リーチ一発があるのでここではフラグをリセットしない

        let pl = &mut self.players[s];
        pl.is_rinshan = true;

        let mut t = tile;
        if t.1 == 0 {
            t.1 = 5; // 赤５を通常の5に変換
        }

        let mut t0 = t;
        if t.1 == 5 && pl.hand[t.0][0] != 0 {
            t0.1 = 0; // 槓した牌で手牌に赤5がある場合
        }

        match meld_type {
            Kakan => {
                let mut idx = 0;
                for m in pl.melds.iter_mut() {
                    if m.tiles[0] == t || m.tiles[1] == t {
                        m.step = self.step;
                        m.type_ = MeldType::Kakan;
                        m.tiles.push(t0);
                        m.froms.push(s);
                        break;
                    }
                    idx += 1;
                }

                let is_shown = pl.is_shown;
                let t1 = if is_shown { t0 } else { Z8 };
                let old = if is_shown { H(s) } else { U };
                self.player_dec_tile(t1);
                self.table_edit(t, old, M(s, idx));
                self.last_tile = Some((s, t0)); // 槍槓フリテン用
            }
            Ankan => {
                let idx = pl.melds.len();
                let mut m = Meld {
                    step: self.step,
                    seat: s,
                    type_: MeldType::Ankan,
                    tiles: vec![],
                    froms: vec![],
                };
                m.tiles = vec![t0, t, t, t];
                m.froms = vec![s, s, s, s];

                let is_shown = pl.is_shown;
                let old = if is_shown { H(s) } else { U };
                for &t1 in &m.tiles {
                    self.player_dec_tile(if is_shown { t1 } else { Z8 });
                }
                for _ in 0..TILE {
                    self.table_edit(t, old, M(s, idx));
                }

                self.players[s].melds.push(m);
            }
            _ => panic!("Invalid meld type"),
        }
    }

    pub fn op_kita(&mut self, seat: Seat, is_drawn: bool) {
        let s = seat;
        let t = Tile(TZ, WN); // z4

        self.step += 1;

        let pl = &mut self.players[s];
        let idx = pl.kitas.len();
        let k = Kita {
            step: self.step,
            seat: s,
            drawn: is_drawn,
        };

        if pl.is_shown {
            self.player_dec_tile(t);
            self.table_edit(t, H(s), K(s, idx));
        } else {
            self.player_dec_tile(Z8);
            self.table_edit(t, U, K(s, idx));
        }

        self.players[s].kitas.push(k);
    }

    pub fn op_roundend_win(
        &mut self,
        _ura_doras: &Vec<Tile>,
        _contexts: &Vec<(Seat, WinContext)>,
        delta_scores: &[i32; SEAT],
    ) {
        self.step += 1;
        self.update_scores(&delta_scores);
    }

    pub fn op_roundend_draw(&mut self, _draw_type: DrawType) {
        self.step += 1;
        // 何もしない
    }

    pub fn op_roundend_notile(&mut self, _is_ready: &[bool; SEAT], delta_scores: &[i32; SEAT]) {
        self.step += 1;
        self.update_scores(&delta_scores);
    }

    #[inline]
    pub fn is_leader(&self, seat: Seat) -> bool {
        seat == self.hand
    }

    #[inline]
    pub fn get_prevalent_wind(&self) -> Tnum {
        self.round % SEAT + 1 // WE | WS | WW | WN
    }

    #[inline]
    pub fn get_seat_wind(&self, seat: Seat) -> Tnum {
        (seat + SEAT - self.hand) % SEAT + 1 // WE | WS | WW | WN
    }

    fn player_inc_tile(&mut self, tile: Tile) {
        let h = &mut self.players[self.turn].hand;
        let t = tile;
        h[t.0][t.1] += 1;
        if t.1 == 0 {
            // 0は赤5のフラグなので本来の5をたてる
            h[t.0][5] += 1;
        }
    }

    fn player_dec_tile(&mut self, tile: Tile) {
        let h = &mut self.players[self.turn].hand;
        let t = tile;
        h[t.0][t.1] -= 1;
        if t.1 == 0 {
            h[t.0][5] -= 1;
        }

        // 5がすべて手牌からなくなた時、赤5フラグをクリア(暗槓用)
        if t.0 != TZ && h[t.0][5] == 0 {
            h[t.0][0] = 0;
        }
    }

    fn disable_ippatsu(&mut self) {
        for s in 0..SEAT {
            self.players[s].is_ippatsu = false;
        }
    }

    fn update_furiten_other(&mut self) {
        if let Some((s, t)) = self.last_tile {
            for s2 in 0..SEAT {
                if s2 != s {
                    if self.players[s2].winning_tiles.contains(&t) {
                        self.players[s2].is_furiten_other = true;
                    }
                }
            }
            self.last_tile = None;
        }
    }

    fn update_scores(&mut self, delta_scores: &[i32; SEAT]) {
        for s in 0..SEAT {
            let mut pl = &mut self.players[s];
            pl.score = pl.score + delta_scores[s];
        }

        let scores = self.players.iter().map(|pl| pl.score).collect();
        let ranks = rank_by_rank_vec(&scores);
        for s in 0..SEAT {
            self.players[s].rank = ranks[s];
        }
    }
}
