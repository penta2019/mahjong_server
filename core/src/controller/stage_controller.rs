use crate::controller::stage_listener::StageListener;
use crate::hand::evaluate::WinContext;
use crate::hand::win::*;
use crate::model::*;
use crate::operator::Operator;
use crate::util::common::rank_by_rank_vec;

use TileStateType::*;

// op実行時にStageListenerすべてに通知するマクロ
macro_rules! op {
    ($self:expr, $name:ident $(, $args:expr)*) => {
        for l in &mut $self.listeners {
            l.$name(&$self.stage, $($args),*);
        }
        for op in &mut $self.operators {
            op.$name(&$self.stage, $($args),*);
        }
    };
}

#[derive(Debug)]
pub struct StageController {
    stage: Stage,
    operators: [Box<dyn Operator>; SEAT],
    listeners: Vec<Box<dyn StageListener>>,
}

impl StageController {
    pub fn new(
        operators: [Box<dyn Operator>; SEAT],
        listeners: Vec<Box<dyn StageListener>>,
    ) -> Self {
        let stage = Stage::default();
        Self {
            stage,
            operators,
            listeners,
        }
    }

    pub fn swap_operator(&mut self, seat: usize, operator: &mut Box<dyn Operator>) {
        std::mem::swap(&mut self.operators[seat], operator);
    }

    pub fn get_stage(&self) -> &Stage {
        return &self.stage;
    }

    pub fn handle_operation(&mut self, seat: Seat, ops: &Vec<PlayerOperation>) -> PlayerOperation {
        self.operators[seat].handle_operation(&self.stage, seat, &ops)
    }

    pub fn op_game_start(&mut self) {
        for s in 0..SEAT {
            self.operators[s].set_seat(s);
        }

        op!(self, notify_op_game_start);
    }

    pub fn op_roundnew(
        &mut self,
        round: usize,
        kyoku: usize,
        honba: usize,
        kyoutaku: usize,
        doras: &Vec<Tile>,
        scores: &[Score; SEAT],
        player_hands: &[Vec<Tile>; SEAT],
    ) {
        self.stage = Stage::default();
        let stg = &mut self.stage;

        stg.round = round;
        stg.kyoku = kyoku;
        stg.honba = honba;
        stg.kyoutaku = kyoutaku;
        stg.step = 1;
        stg.left_tile_count = 69;

        stg.tile_remains = [[TILE; TNUM]; TYPE];
        let r = &mut stg.tile_remains;
        r[TM][0] = 1;
        r[TP][0] = 1;
        r[TS][0] = 1;
        r[TZ][0] = 0;

        for s in 0..SEAT {
            let ph = &player_hands[s];
            let pl = &mut stg.players[s];
            pl.seat = s;
            pl.is_shown = !ph.is_empty() && !ph.contains(&Z8);
            pl.is_menzen = true;

            stg.turn = s; // player_inc_tile() 用
            if pl.is_shown {
                if s == kyoku {
                    pl.drawn = Some(*ph.last().unwrap());
                }
                for &t in ph {
                    player_inc_tile(stg, t);
                    table_edit(stg, t, U, H(s));
                }
            } else {
                if s == kyoku {
                    pl.drawn = Some(Z8);
                }
                pl.hand[TZ][UK] = if s == kyoku { 14 } else { 13 }; // 親:14, 子:13
            }
        }
        update_scores(stg, scores);
        stg.turn = kyoku;

        for &d in doras {
            table_edit(stg, d, U, R);
        }
        stg.doras = doras.clone();

        op!(
            self,
            notify_op_roundnew,
            round,
            kyoku,
            honba,
            kyoutaku,
            doras,
            scores,
            player_hands
        );
    }

    pub fn op_dealtile(&mut self, seat: Seat, tile: Tile) {
        // tileはplayer.is_shown = falseの場合,Z8になることに注意
        let stg = &mut self.stage;
        update_after_discard_completed(stg);

        let s = seat;

        stg.step += 1;
        stg.turn = s;
        stg.left_tile_count -= 1;

        if stg.players[s].is_rinshan {
            // 槍槓リーチ一発を考慮して加槓の成立が確定したタイミングで一発フラグをリセット
            disable_ippatsu(stg);
        }

        if tile != Z8 {
            table_edit(stg, tile, U, H(s));
        }
        player_inc_tile(stg, tile);
        stg.players[s].drawn = Some(tile);

        op!(self, notify_op_dealtile, seat, tile);
    }

    pub fn op_discardtile(&mut self, seat: Seat, tile: Tile, is_drawn: bool, is_riichi: bool) {
        let stg = &mut self.stage;
        let s = seat;
        let mut t = tile;
        let no_meld = stg.players.iter().all(|pl| pl.melds.is_empty());

        stg.step += 1;
        stg.turn = s;

        let pl = &mut stg.players[s];
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
            if is_riichi && no_meld && pl.discards.is_empty() {
                pl.is_daburii = true;
            }
            stg.last_riichi = Some(s);
        } else {
            pl.is_ippatsu = false;
        }

        let idx = pl.discards.len();
        let d = Discard {
            step: stg.step,
            tile: t,
            drawn: is_drawn,
            meld: None,
        };

        if pl.is_shown {
            player_dec_tile(stg, t);
            table_edit(stg, t, H(s), D(s, idx));
        } else {
            player_dec_tile(stg, Z8);
            table_edit(stg, t, U, D(s, idx));
        }

        stg.discards.push((s, idx));
        stg.players[s].discards.push(d);

        // 和了牌とフリテンの計算
        let pl = &mut stg.players[s];
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
                pl.win_tiles = dedup;
            } else if pl.win_tiles.contains(&t) {
                // 和了牌をツモ切り(役無しまたは点数状況で和了れない場合など)
                pl.is_furiten = true;
            }
        }
        pl.drawn = None;

        stg.last_tile = Some((s, OpType::Discard, t));

        op!(self, notify_op_discardtile, seat, tile, is_drawn, is_riichi);
    }

    pub fn op_chiponkan(
        &mut self,
        seat: Seat,
        meld_type: MeldType,
        tiles: &Vec<Tile>,
        froms: &Vec<Seat>,
    ) {
        let stg = &mut self.stage;
        update_after_discard_completed(stg);

        let s = seat;
        stg.step += 1;
        stg.turn = s;
        disable_ippatsu(stg);

        let pl = &mut stg.players[s];
        pl.is_menzen = false;
        if meld_type == MeldType::Minkan {
            pl.is_rinshan = true;
        }

        let idx = pl.melds.len();
        let m = Meld {
            step: stg.step,
            seat: s,
            type_: meld_type,
            tiles: tiles.clone(),
            froms: froms.clone(),
        };
        let &(prev_s, prev_i) = stg.discards.last().unwrap();
        stg.players[prev_s].discards[prev_i].meld = Some((s, idx));

        for (&t, &f) in m.tiles.iter().zip(m.froms.iter()) {
            if f == s {
                if stg.players[s].is_shown {
                    player_dec_tile(stg, t);
                    table_edit(stg, t, H(s), M(s, idx));
                } else {
                    player_dec_tile(stg, Z8);
                    table_edit(stg, t, U, M(s, idx));
                }
            }
        }

        stg.players[s].melds.push(m);

        op!(self, notify_op_chiponkan, seat, meld_type, tiles, froms);
    }

    pub fn op_ankankakan(&mut self, seat: Seat, meld_type: MeldType, tile: Tile) {
        let stg = &mut self.stage;
        let s = seat;

        stg.step += 1;
        // self.disable_ippatsu(); 槍槓リーチ一発があるのでここではフラグをリセットしない

        let pl = &mut stg.players[s];
        pl.is_rinshan = true;

        let mut t = tile;
        if t.1 == 0 {
            t.1 = 5; // 赤５を通常の5に変換
        }

        match meld_type {
            MeldType::Ankan => {
                let idx = pl.melds.len();
                let mut t0 = t;
                if t.is_suit() && t.1 == 5 {
                    t0.1 = 0; // 赤5
                }
                let m = Meld {
                    step: stg.step,
                    seat: s,
                    type_: MeldType::Ankan,
                    tiles: vec![t0, t, t, t],
                    froms: vec![s, s, s, s],
                };
                let is_shown = pl.is_shown;
                let old = if is_shown { H(s) } else { U };
                for &t1 in &m.tiles {
                    player_dec_tile(stg, if is_shown { t1 } else { Z8 });
                }
                for _ in 0..TILE {
                    table_edit(stg, t, old, M(s, idx));
                }
                stg.players[s].melds.push(m);
                if t.is_end() {
                    // 国士の暗槓ロン
                    stg.last_tile = Some((s, OpType::Ankan, t));
                }
            }
            MeldType::Kakan => {
                let mut idx = 0;
                for m in pl.melds.iter_mut() {
                    if m.tiles[0] == t || m.tiles[1] == t {
                        m.step = stg.step;
                        m.type_ = MeldType::Kakan;
                        m.tiles.push(tile);
                        m.froms.push(s);
                        break;
                    }
                    idx += 1;
                }

                let is_shown = pl.is_shown;
                let t1 = if is_shown { tile } else { Z8 };
                let old = if is_shown { H(s) } else { U };
                player_dec_tile(stg, t1);
                table_edit(stg, t, old, M(s, idx));
                stg.last_tile = Some((s, OpType::Kakan, tile)); // 槍槓フリテン用
            }
            _ => panic!("Invalid meld type"),
        }

        op!(self, notify_op_ankankakan, seat, meld_type, tile);
    }

    pub fn op_kita(&mut self, seat: Seat, is_drawn: bool) {
        let stg = &mut self.stage;
        let s = seat;
        let t = Tile(TZ, WN); // z4

        stg.step += 1;

        let pl = &mut stg.players[s];
        let idx = pl.kitas.len();
        let k = Kita {
            step: stg.step,
            seat: s,
            drawn: is_drawn,
        };

        if pl.is_shown {
            player_dec_tile(stg, t);
            table_edit(stg, t, H(s), K(s, idx));
        } else {
            player_dec_tile(stg, Z8);
            table_edit(stg, t, U, K(s, idx));
        }

        stg.players[s].kitas.push(k);

        op!(self, notify_op_kita, seat, is_drawn);
    }

    pub fn op_dora(&mut self, tile: Tile) {
        let stg = &mut self.stage;
        table_edit(stg, tile, TileStateType::U, TileStateType::R);
        stg.doras.push(tile);

        op!(self, notify_op_dora, tile);
    }

    pub fn op_roundend_win(
        &mut self,
        ura_doras: &Vec<Tile>,
        contexts: &Vec<(Seat, [Point; SEAT], WinContext)>,
    ) {
        let stg = &mut self.stage;
        stg.step += 1;
        for ctx in contexts {
            update_scores(stg, &ctx.1);
        }

        op!(self, notify_op_roundend_win, ura_doras, contexts);
    }

    pub fn op_roundend_draw(&mut self, draw_type: DrawType) {
        let stg = &mut self.stage;
        stg.step += 1;

        op!(self, notify_op_roundend_draw, draw_type);
    }

    pub fn op_roundend_notile(&mut self, is_tenpai: &[bool; SEAT], points: &[Point; SEAT]) {
        let stg = &mut self.stage;
        stg.step += 1;
        update_scores(stg, &points);

        op!(self, notify_op_roundend_notile, is_tenpai, points);
    }

    pub fn op_game_over(&mut self) {
        op!(self, notify_op_game_over);
    }
}

fn table_edit(stg: &mut Stage, tile: Tile, old: TileStateType, new: TileStateType) {
    let te = &mut stg.tile_states[tile.0][tile.n()];
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
        stg.tile_remains[tile.0][tile.n()] -= 1;
        if tile.1 == 0 {
            stg.tile_remains[tile.0][0] -= 1;
        }
    }
}

fn player_inc_tile(stg: &mut Stage, tile: Tile) {
    let h = &mut stg.players[stg.turn].hand;
    let t = tile;
    h[t.0][t.1] += 1;
    if t.1 == 0 {
        // 0は赤5のフラグなので本来の5をたてる
        h[t.0][5] += 1;
    }
}

fn player_dec_tile(stg: &mut Stage, tile: Tile) {
    let h = &mut stg.players[stg.turn].hand;
    let t = tile;
    h[t.0][t.1] -= 1;
    if t.1 == 0 {
        h[t.0][5] -= 1;
    }

    // 5がすべて手牌からなくなた時,赤5フラグをクリア(暗槓用)
    if t.is_suit() && h[t.0][5] == 0 {
        h[t.0][0] = 0;
    }
}

fn disable_ippatsu(stg: &mut Stage) {
    for s in 0..SEAT {
        stg.players[s].is_ippatsu = false;
    }
}

fn update_after_discard_completed(stg: &mut Stage) {
    // 他のプレイヤーの捨て牌,または加槓した牌の見逃しフリテン
    if let Some((s, tp, t)) = stg.last_tile {
        if tp == OpType::Discard || tp == OpType::Kakan {
            for s2 in 0..SEAT {
                if s2 != s {
                    if stg.players[s2].win_tiles.contains(&t) {
                        stg.players[s2].is_furiten_other = true;
                    }
                }
            }
        }
    }

    // リーチがロンされずに成立した場合の供託への点棒追加
    if let Some(s) = stg.last_riichi {
        stg.players[s].score -= 1000;
        stg.kyoutaku += 1;
        stg.last_riichi = None;
    }
}

fn update_scores(stg: &mut Stage, points: &[Point; SEAT]) {
    for s in 0..SEAT {
        let mut pl = &mut stg.players[s];
        pl.score = pl.score + points[s];
    }

    let scores = stg.players.iter().map(|pl| pl.score).collect();
    let ranks = rank_by_rank_vec(&scores);
    for s in 0..SEAT {
        stg.players[s].rank = ranks[s];
    }
}
