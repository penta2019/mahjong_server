use super::*;
use crate::hand::evaluate::WinContext;
use crate::hand::win::*;
use crate::util::common::{rank_by_rank_vec, vec_to_string};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DrawType {
    Kyushukyuhai,   // 九種九牌
    Suufuurenda,    // 四風連打
    Suukansanra,    // 四槓散了
    Suuchariichi,   // 四家立直
    Kouhaiheikyoku, // 荒廃平局
}

#[derive(Debug, Default, Serialize)]
pub struct Stage {
    pub round: usize,                    // 場 (東:0, 南:1, 西:2, 北:3)
    pub kyoku: usize,                    // 局 (0~3 = 親のseat)
    pub honba: usize,                    // 本場
    pub kyoutaku: usize,                 // リーチ棒の供託
    pub turn: Seat,                      // 順番
    pub step: usize,                     // ステップ op関数を呼び出す毎に+1する
    pub left_tile_count: usize,          // 牌山の残り枚数
    pub doras: Vec<Tile>,                // ドラ表示牌
    pub discards: Vec<(Seat, Index)>,    // プレイヤー全員の捨て牌
    pub last_tile: Option<(Seat, Tile)>, // 他家にロンされる可能性のある牌(捨て牌,槍槓) フリテン判定用
    pub last_riichi: Option<Seat>,       // リーチがロンされずに成立した場合の供託更新用
    pub players: [Player; SEAT],         // 各プレイヤー情報
    pub is_3p: bool,                     // 三麻フラグ(未実装, 常にfalse)
    pub tile_states: [[[TileStateType; TILE]; TNUM]; TYPE],
    pub tile_remains: [[usize; TNUM]; TYPE], // 牌の残り枚数 = 山+手牌(捨て牌,副露牌,ドラ表示牌以外)
}

impl Stage {
    pub fn new() -> Self {
        let mut stg = Self::default();
        stg.tile_remains = [[TILE; TNUM]; TYPE];
        let rem = &mut stg.tile_remains;
        rem[TM][0] = 1;
        rem[TP][0] = 1;
        rem[TS][0] = 1;
        rem[TZ][0] = 0;
        stg
    }

    pub fn print(&self) {
        println!(
            "round: {}, hand: {}, honba: {}, kyoutaku: {}\n\
            turn: {}, left_tile_count: {}, doras: {}, last_tile: {:?}",
            self.round,
            self.kyoku,
            self.honba,
            self.kyoutaku,
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

        println!("remaining tiles");
        for ti in 0..TYPE {
            println!("{}: {:?}", ['m', 'p', 's', 'z'][ti], self.tile_remains[ti]);
        }
        println!();
    }

    pub fn get_scores(&self) -> [i32; SEAT] {
        let mut scores = [0; SEAT];
        for s in 0..SEAT {
            scores[s] = self.players[s].score;
        }
        scores
    }

    fn table_edit(&mut self, tile: Tile, old: TileStateType, new: TileStateType) {
        let te = &mut self.tile_states[tile.0][tile.n()];
        // println!("[table_edit] {}: {:?} | {:?} => {:?}", tile, te, old, new);
        let i = te.iter().position(|&x| x == old).unwrap();
        te[i] = new.clone();
        te.sort();

        if match old {
            U => true,
            H(_) => true,
            _ => false,
        } && match new {
            H(_) => false,
            _ => true,
        } {
            self.tile_remains[tile.0][tile.n()] -= 1;
            if tile.1 == 0 {
                self.tile_remains[tile.0][0] -= 1;
            }
        }
    }

    pub fn op_game_start(&mut self) {}

    pub fn op_roundnew(
        &mut self,
        round: usize,
        kyoku: usize,
        honba: usize,
        kyoutaku: usize,
        doras: &Vec<Tile>,
        scores: &[i32; SEAT],
        player_hands: &[Vec<Tile>; SEAT],
    ) {
        std::mem::swap(self, &mut Self::new());

        self.round = round;
        self.kyoku = kyoku;
        self.honba = honba;
        self.kyoutaku = kyoutaku;
        self.step = 1;
        self.left_tile_count = 69;

        for s in 0..SEAT {
            let ph = &player_hands[s];
            let pl = &mut self.players[s];
            pl.seat = s;
            pl.is_shown = !ph.is_empty() && !ph.contains(&Z8);
            pl.is_menzen = true;

            self.turn = s; // player_inc_tile() 用
            if pl.is_shown {
                if s == kyoku {
                    pl.drawn = Some(*ph.last().unwrap());
                }
                for &t in ph {
                    self.player_inc_tile(t);
                    self.table_edit(t, U, H(s));
                }
            } else {
                if s == kyoku {
                    pl.drawn = Some(Z8);
                }
                pl.hand[TZ][UK] = if s == kyoku { 14 } else { 13 }; // 親:14, 子:13
            }
        }
        self.update_scores(scores);
        self.turn = kyoku;

        for &d in doras {
            self.table_edit(d, U, R);
        }
        self.doras = doras.clone();
    }

    pub fn op_dealtile(&mut self, seat: Seat, tile: Option<Tile>) {
        self.update_after_discard_completed();

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
        let mut t = tile;
        let riichi_no_meld = is_riichi && self.players.iter().all(|pl| pl.melds.is_empty());

        self.step += 1;
        self.turn = s;

        let pl = &mut self.players[s];
        pl.is_rinshan = false;
        pl.is_furiten_other = false;

        // 5の牌を捨てようとしたとき赤5の牌しか手牌にない場合は赤5に変換
        if t.1 == 5 && pl.hand[t.0][5] == 1 && pl.hand[t.0][0] == 1 {
            t = Tile(t.0, 0);
        }

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
            self.last_riichi = Some(s);
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
                // 和了牌をツモ切り(役無しまたは点数状況で和了れない場合など)
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
        self.update_after_discard_completed();

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

        for (&t, &f) in m.tiles.iter().zip(m.froms.iter()) {
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
            MeldType::Kakan => {
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
            MeldType::Ankan => {
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

    pub fn op_dora(&mut self, tile: Tile) {
        self.table_edit(tile, TileStateType::U, TileStateType::R);
        self.doras.push(tile);
    }

    pub fn op_roundend_win(
        &mut self,
        _ura_doras: &Vec<Tile>,
        contexts: &Vec<(Seat, [i32; SEAT], WinContext)>,
    ) {
        self.step += 1;
        for ctx in contexts {
            self.update_scores(&ctx.1);
        }
    }

    pub fn op_roundend_draw(&mut self, _draw_type: DrawType) {
        self.step += 1;
        // 何もしない
    }

    pub fn op_roundend_notile(&mut self, _is_ready: &[bool; SEAT], delta_scores: &[i32; SEAT]) {
        self.step += 1;
        self.update_scores(&delta_scores);
    }

    pub fn op_game_over(&mut self) {}

    #[inline]
    pub fn is_leader(&self, seat: Seat) -> bool {
        seat == self.kyoku
    }

    #[inline]
    pub fn get_prevalent_wind(&self) -> Tnum {
        self.round % SEAT + 1 // WE | WS | WW | WN
    }

    #[inline]
    pub fn get_seat_wind(&self, seat: Seat) -> Tnum {
        (seat + SEAT - self.kyoku) % SEAT + 1 // WE | WS | WW | WN
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
        if t.is_suit() && h[t.0][5] == 0 {
            h[t.0][0] = 0;
        }
    }

    fn disable_ippatsu(&mut self) {
        for s in 0..SEAT {
            self.players[s].is_ippatsu = false;
        }
    }

    fn update_after_discard_completed(&mut self) {
        // 他のプレイヤーの捨て牌、または加槓した牌の見逃しフリテン
        if let Some((s, t)) = self.last_tile {
            for s2 in 0..SEAT {
                if s2 != s {
                    if self.players[s2].winning_tiles.contains(&t) {
                        self.players[s2].is_furiten_other = true;
                    }
                }
            }
        }

        // リーチがロンされずに成立した場合の供託への点棒追加
        if let Some(s) = self.last_riichi {
            self.players[s].score -= 1000;
            self.kyoutaku += 1;
            self.last_riichi = None;
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
