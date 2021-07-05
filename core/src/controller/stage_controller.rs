use std::fmt;

use crate::hand::win::*;
use crate::model::*;
use crate::operator::Operator;
use crate::util::common::rank_by_rank_vec;

use TileStateType::*;

// [StageListener (Observer Pattern)]
pub trait StageListener: Send {
    fn notify_action(&mut self, _stg: &Stage, _act: &Action) {}
}

impl fmt::Debug for dyn StageListener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StageListener")
    }
}

// [StageController]
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

    pub fn handle_action(&mut self, act: &Action) {
        let stg = &mut self.stage;
        stg.step += 1;
        match act {
            Action::GameStart(a) => {
                for s in 0..SEAT {
                    self.operators[s].set_seat(s);
                }
                action_game_start(stg, a);
            }
            Action::RoundNew(a) => action_round_new(stg, a),
            Action::DealTile(a) => action_deal_tile(stg, a),
            Action::DiscardTile(a) => action_discard_tile(stg, a),
            Action::Meld(a) => action_meld(stg, a),
            Action::Kita(a) => action_kita(stg, a),
            Action::Dora(a) => action_dora(stg, a),
            Action::RoundEndWin(a) => action_round_end_win(stg, a),
            Action::RoundEndDraw(a) => action_round_end_draw(stg, a),
            Action::RoundEndNoTile(a) => action_round_end_no_tile(stg, a),
            Action::GameOver(a) => action_game_over(stg, a),
        }

        for l in &mut self.operators {
            l.notify_action(stg, &act);
        }
        for l in &mut self.listeners {
            l.notify_action(stg, &act);
        }
    }

    pub fn handle_operation(&mut self, seat: Seat, ops: &Vec<PlayerOperation>) -> PlayerOperation {
        self.operators[seat].handle_operation(&self.stage, seat, &ops)
    }
}

// [Action]
fn action_game_start(_stg: &mut Stage, _act: &ActionGameStart) {}

fn action_round_new(stg: &mut Stage, act: &ActionRoundNew) {
    *stg = Stage::default();
    stg.round = act.round;
    stg.kyoku = act.kyoku;
    stg.honba = act.honba;
    stg.kyoutaku = act.kyoutaku;
    stg.turn = act.kyoku;
    stg.left_tile_count = 69;
    stg.doras = act.doras.clone();
    stg.tile_remains = [[TILE; TNUM]; TYPE];
    let r = &mut stg.tile_remains;
    r[TM][0] = 1;
    r[TP][0] = 1;
    r[TS][0] = 1;
    r[TZ][0] = 0;
    update_scores(stg, &act.scores);

    // プレイヤー情報
    for s in 0..SEAT {
        let ph = &act.hands[s];
        let pl = &mut stg.players[s];
        pl.seat = s;
        pl.is_shown = !ph.is_empty() && !ph.contains(&Z8);
        pl.is_menzen = true;

        if pl.is_shown {
            let mut drawn = None; // 親番14枚目
            let last = if s == act.kyoku {
                drawn = ph.last();
                ph.len() - 1
            } else {
                ph.len()
            };
            for &t in &ph[..last] {
                player_inc_tile(pl, t);
            }

            let pl = &mut stg.players[s];
            pl.win_tiles = get_win_tiles(pl);
            if let Some(&t) = drawn {
                pl.drawn = Some(t);
                player_inc_tile(pl, t);
            }
        } else {
            if s == act.kyoku {
                pl.drawn = Some(Z8);
            }
            pl.hand[TZ][UK] = if s == act.kyoku { 14 } else { 13 }; // 親:14, 子:13
        }
    }

    // update tile_states
    for &d in &act.doras {
        table_edit(stg, d, U, R);
    }
    for s in 0..SEAT {
        let ph = &act.hands[s];
        if stg.players[s].is_shown {
            for &t in ph {
                table_edit(stg, t, U, H(s));
            }
        }
    }
}

fn action_deal_tile(stg: &mut Stage, act: &ActionDealTile) {
    let s = act.seat;
    let t = act.tile; // tileはplayer.is_shown = falseの場合,Z8になることに注意

    update_after_discard_completed(stg);
    if stg.players[s].is_rinshan {
        // 槍槓リーチ一発を考慮して加槓の成立が確定したタイミングで一発フラグをリセット
        disable_ippatsu(stg);
    }

    stg.turn = s;
    stg.left_tile_count -= 1;
    stg.players[s].drawn = Some(t);
    player_inc_tile(&mut stg.players[s], t);
    if t != Z8 {
        table_edit(stg, t, U, H(s));
    }
}

fn action_discard_tile(stg: &mut Stage, act: &ActionDiscardTile) {
    let s = act.seat;
    let mut t = act.tile;
    let no_meld = stg.players.iter().all(|pl| pl.melds.is_empty());

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
                act.is_drawn
            }
        } else {
            false
        }
    } else {
        act.is_drawn
    };
    if act.is_riichi {
        assert!(pl.riichi == None);
        pl.riichi = Some(pl.discards.len());
        pl.is_riichi = true;
        pl.is_ippatsu = true;
        if no_meld && pl.discards.is_empty() {
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

    stg.discards.push((s, idx));
    pl.discards.push(d);

    if pl.is_shown {
        player_dec_tile(pl, t);
        table_edit(stg, t, H(s), D(s, idx));
    } else {
        player_dec_tile(pl, Z8);
        table_edit(stg, t, U, D(s, idx));
    }

    // 和了牌とフリテンの計算
    let pl = &mut stg.players[s];
    if pl.is_shown {
        if pl.drawn != Some(t) {
            let wt = get_win_tiles(pl);
            pl.is_furiten = false;
            if !wt.is_empty() {
                let mut tt = TileTable::default();
                for w in &wt {
                    tt[w.0][w.1] = 1;
                }
                for d in &pl.discards {
                    let dt = d.tile.to_normal();
                    if tt[dt.0][dt.1] != 0 {
                        pl.is_furiten = true;
                        break;
                    }
                }
            }
            pl.win_tiles = wt;
        } else if pl.win_tiles.contains(&t) {
            // 和了牌をツモ切り(役無しまたは点数状況で和了れない場合など)
            pl.is_furiten = true;
        }
    }

    pl.drawn = None;
    stg.last_tile = Some((s, OpType::Discard, t));
}

fn action_meld(stg: &mut Stage, act: &ActionMeld) {
    // リーチ一発や槍槓, フリテンなどに必要な前処理
    update_after_discard_completed(stg);
    if act.meld_type != MeldType::Kakan {
        disable_ippatsu(stg); // 加槓の場合は槍槓リーチ一発があるので一旦スルー
    }

    let s = act.seat;
    stg.turn = s;
    let pl = &mut stg.players[s];
    let mut idx; // Vec<Meld>のインデックス
    match act.meld_type {
        MeldType::Chi | MeldType::Pon | MeldType::Minkan => {
            let lt = stg.last_tile.unwrap();
            assert!(lt.1 == OpType::Discard);

            pl.is_menzen = false;
            if act.meld_type == MeldType::Minkan {
                pl.is_rinshan = true;
            }
            idx = pl.melds.len();
            let mut tiles = act.consumed.clone();
            let mut froms = vec![s; tiles.len()];
            tiles.push(lt.2);
            froms.push(lt.0);
            let m = Meld {
                step: stg.step,
                seat: s,
                type_: act.meld_type,
                tiles: tiles,
                froms: froms,
            };
            let &(prev_s, prev_i) = stg.discards.last().unwrap();
            stg.players[prev_s].discards[prev_i].meld = Some((s, idx));
            stg.players[s].melds.push(m);
        }
        MeldType::Ankan => {
            pl.is_rinshan = true;
            idx = pl.melds.len();
            let tiles = act.consumed.clone();
            let froms = vec![s; tiles.len()];
            let m = Meld {
                step: stg.step,
                seat: s,
                type_: MeldType::Ankan,
                tiles: tiles,
                froms: froms,
            };
            pl.melds.push(m);

            let t = act.consumed[0];
            if t.is_end() {
                // 国士の暗槓ロン
                stg.last_tile = Some((s, OpType::Ankan, t));
            }
        }
        MeldType::Kakan => {
            // disable_ippatsu(stg); // 槍槓リーチ一発があるのでここではフラグをリセットしない
            pl.is_rinshan = true;
            idx = 0;
            let t = act.consumed[0];
            for m in pl.melds.iter_mut() {
                if m.tiles[0] == t || m.tiles[1] == t {
                    m.step = stg.step;
                    m.type_ = MeldType::Kakan;
                    m.tiles.push(t);
                    m.froms.push(s);
                    break;
                }
                idx += 1;
            }

            stg.last_tile = Some((s, OpType::Kakan, t)); // 槍槓+フリテン用
        }
    }

    for &t in &act.consumed {
        let pl = &mut stg.players[s];
        if pl.is_shown {
            player_dec_tile(pl, t);
            table_edit(stg, t, H(s), M(s, idx));
        } else {
            player_dec_tile(pl, Z8);
            table_edit(stg, t, U, M(s, idx));
        }
    }
}

fn action_kita(stg: &mut Stage, act: &ActionKita) {
    let s = act.seat;
    let t = Tile(TZ, WN); // z4
    let pl = &mut stg.players[s];
    let idx = pl.kitas.len();
    let k = Kita {
        step: stg.step,
        seat: s,
        drawn: act.is_drawn,
    };

    if pl.is_shown {
        player_dec_tile(pl, t);
        table_edit(stg, t, H(s), K(s, idx));
    } else {
        player_dec_tile(pl, Z8);
        table_edit(stg, t, U, K(s, idx));
    }

    stg.players[s].kitas.push(k);
}

fn action_dora(stg: &mut Stage, act: &ActionDora) {
    table_edit(stg, act.tile, TileStateType::U, TileStateType::R);
    stg.doras.push(act.tile);
}

fn action_round_end_win(stg: &mut Stage, act: &ActionRoundEndWin) {
    for ctx in &act.contexts {
        update_scores(stg, &ctx.1);
    }
}

fn action_round_end_draw(_stg: &mut Stage, _act: &ActionRoundEndDraw) {}

fn action_round_end_no_tile(stg: &mut Stage, act: &ActionRoundEndNoTile) {
    update_scores(stg, &act.points);
}

fn action_game_over(_stg: &mut Stage, _act: &ActionGameOver) {}

// [Utility]
fn table_edit(stg: &mut Stage, tile: Tile, old: TileStateType, new: TileStateType) {
    let tn = tile.to_normal();
    let te = &mut stg.tile_states[tn.0][tn.1];
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
        stg.tile_remains[tn.0][tn.1] -= 1;
        if tile.1 == 0 {
            stg.tile_remains[tile.0][tile.1] -= 1;
        }
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

fn player_inc_tile(pl: &mut Player, tile: Tile) {
    let h = &mut pl.hand;
    let t = tile;
    h[t.0][t.1] += 1;
    if t.1 == 0 {
        // 0は赤5のフラグなので本来の5をたてる
        h[t.0][5] += 1;
    }
}

fn player_dec_tile(pl: &mut Player, tile: Tile) {
    let h = &mut pl.hand;
    let t = tile;
    h[t.0][t.1] -= 1;
    if t.1 == 0 {
        h[t.0][5] -= 1;
    }
    assert!(h[t.0][5] != 0 || h[t.0][0] == 0);
}

fn get_win_tiles(pl: &Player) -> Vec<Tile> {
    let mut win_tiles = vec![];
    let mut tt = TileTable::default();
    let wts0 = calc_tiles_to_kokushimusou_win(&pl.hand);
    let wts1 = calc_tiles_to_normal_win(&pl.hand);
    let wts2 = calc_tiles_to_chiitoitsu_win(&pl.hand);
    for wts in &[wts0, wts1, wts2] {
        for &t in wts {
            if tt[t.0][t.1] == 0 {
                tt[t.0][t.1] += 1;
                win_tiles.push(t);
            }
        }
    }
    win_tiles
}
