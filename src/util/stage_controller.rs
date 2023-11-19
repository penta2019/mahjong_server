use crate::actor::Actor;
use crate::etc::misc::rank_by_rank_vec;
use crate::hand::*;
use crate::listener::Listener;
use crate::model::*;
use crate::util::common::*;

use TileState::*;

#[derive(Debug)]
pub struct StageController {
    stage: Stage,
    actors: [Box<dyn Actor>; SEAT],
    listeners: Vec<Box<dyn Listener>>,
}

impl StageController {
    pub fn new(actors: [Box<dyn Actor>; SEAT], listeners: Vec<Box<dyn Listener>>) -> Self {
        let stage = Stage::default();
        Self {
            stage,
            actors,
            listeners,
        }
    }

    pub fn swap_actor(&mut self, seat: usize, actor: &mut Box<dyn Actor>) {
        std::mem::swap(&mut self.actors[seat], actor);
    }

    pub fn get_stage(&self) -> &Stage {
        &self.stage
    }

    pub fn get_names(&self) -> [String; SEAT] {
        let mut names = [
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        ];
        for s in 0..SEAT {
            for actor in &self.actors {
                names[s] = actor.get_name();
            }
        }
        names
    }

    pub fn handle_event(&mut self, event: &Event) {
        if let Event::New(_) = event {
            for s in 0..SEAT {
                self.actors[s].init(s);
            }
        }

        let stg = &mut self.stage;
        match event {
            Event::Begin(e) => event_begin(stg, e),
            Event::New(e) => event_new(stg, e),
            Event::Deal(e) => event_deal(stg, e),
            Event::Discard(e) => event_discard(stg, e),
            Event::Meld(e) => event_meld(stg, e),
            Event::Nukidora(e) => event_kita(stg, e),
            Event::Dora(e) => event_dora(stg, e),
            Event::Win(e) => event_win(stg, e),
            Event::Draw(e) => event_draw(stg, e),
            Event::End(e) => event_end(stg, e),
        }

        // Actorより先にListenrsにイベントを通知
        // Debug(Listener)などが一時停止する可能性があるため, またActorが特定のイベントでクラッシュする際にイベントを前もって補足するため
        for a in &mut self.listeners {
            a.notify_event(stg, event);
        }
        for a in &mut self.actors {
            a.notify_event(stg, event);
        }

        stg.step += 1;
    }

    pub fn select_action(
        &mut self,
        seat: Seat,
        acts: &Vec<Action>,
        tenpais: &Vec<Tenpai>,
        retry: i32,
    ) -> Option<Action> {
        let res = self.actors[seat].select_action(&self.stage, acts, tenpais, retry);

        // turn operationの際に聴牌情報を参照して和了牌を更新
        if let Some(act) = &res {
            use ActionType::*;
            let pl = &mut self.stage.players[seat];
            let t = match act.action_type {
                Discard | Riichi | Kakan | Ankan | Nukidora => {
                    // Ankanは手牌から4枚なくなるため例外的ではあるが和了牌が増えることはないので同様に処理
                    if act.tiles.is_empty() {
                        pl.drawn.unwrap() // ツモ切りリーチ
                    } else {
                        act.tiles[0]
                    }
                }
                _ => Z8,
            };
            if t != Z8 && Some(t) != pl.drawn {
                pl.winning_tiles = vec![];
                for tp in tenpais {
                    if tp.discard_tile == t {
                        for wt in &tp.winning_tiles {
                            pl.winning_tiles.push(wt.tile);
                        }
                        pl.is_furiten = tp.is_furiten;
                        break;
                    }
                }
            }
        }

        res
    }
}

// [Event]
fn event_begin(_stg: &mut Stage, _event: &EventBegin) {}

fn event_new(stg: &mut Stage, event: &EventNew) {
    *stg = Stage::default();
    stg.rule = event.rule.clone();
    stg.round = event.round;
    stg.dealer = event.dealer;
    stg.honba_sticks = event.honba_sticks;
    stg.riichi_sticks = event.riichi_sticks;
    stg.turn = (event.dealer + 3) % SEAT; // 親の14枚目(ツモ)でturn=dealerになる
    stg.wall_count = event.wall_count;
    stg.doras = event.doras.clone();
    update_scores(stg, &event.scores);

    // プレイヤー情報
    for s in 0..SEAT {
        let ph = &event.hands[s];
        let pl = &mut stg.players[s];
        pl.seat = s;
        pl.is_shown = !ph.is_empty() && !ph.contains(&Z8);
        pl.is_menzen = true;
        pl.is_nagashimangan = true;

        if pl.is_shown {
            for &t in ph {
                player_inc_tile(pl, t);
            }
            let pl = &mut stg.players[s];
            pl.winning_tiles = get_winning_tiles(pl);
        } else {
            // 手牌が見えない場合,牌すべてz8(不明な牌)になる
            pl.hand[TZ][UK] = 13;
        }
    }

    // update tile_states
    for &d in &event.doras {
        table_edit(stg, d, U, R);
    }
    for s in 0..SEAT {
        let ph = &event.hands[s];
        if stg.players[s].is_shown {
            for &t in ph {
                table_edit(stg, t, U, H(s));
            }
        }
    }
}

fn event_deal(stg: &mut Stage, event: &EventDeal) {
    let s = event.seat;
    let t = event.tile; // tileはplayer.is_shown = falseの場合,Z8になることに注意

    update_after_discard_completed(stg);

    if stg.players[s].is_rinshan {
        // 槍槓リーチ一発を考慮して加槓の成立が確定したタイミングで一発フラグをリセット
        disable_ippatsu(stg);
    }

    // call_operation用なのでturn_operationでセットして
    // ここ(call_operation終了後)でNoneをセット
    stg.last_tile = None;

    stg.turn = s;
    stg.wall_count -= 1;

    let pl = &mut stg.players[s];
    pl.drawn = Some(t);
    player_inc_tile(pl, t);
    if !pl.is_riichi {
        pl.is_furiten_other = false; // リーチ中でなければ見逃しフリテンを解除
    }

    if t != Z8 {
        table_edit(stg, t, U, H(s));
    }
}

fn event_discard(stg: &mut Stage, event: &EventDiscard) {
    let s = event.seat;
    let t = event.tile;
    let no_meld = stg.players.iter().all(|pl| pl.melds.is_empty());

    stg.turn = s;
    let pl = &mut stg.players[s];
    pl.is_rinshan = false;
    if t.is_simple() {
        pl.is_nagashimangan = false;
    }
    if pl.is_shown {
        assert!(count_tile(&pl.hand, t) > 0, "{} not found in hand", t);
    }

    let is_drop = if pl.is_shown {
        if pl.drawn == Some(t) {
            if pl.hand[t.0][t.1] == 1 {
                true
            } else {
                event.is_drop
            }
        } else {
            false
        }
    } else {
        event.is_drop
    };
    if event.is_riichi {
        assert!(pl.riichi.is_none());
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
        is_drop,
        meld: None,
    };

    stg.discards.push((s, idx));
    pl.discards.push(d);
    pl.drawn = None;

    if pl.is_shown {
        player_dec_tile(pl, t);
        table_edit(stg, t, H(s), D(s, idx));
    } else {
        player_dec_tile(pl, Z8);
        table_edit(stg, t, U, D(s, idx));
    }

    stg.last_tile = Some((s, ActionType::Discard, t));
}

fn event_meld(stg: &mut Stage, event: &EventMeld) {
    // リーチ一発や槍槓, フリテンなどに必要な前処理
    update_after_discard_completed(stg);
    if event.meld_type != MeldType::Kakan {
        disable_ippatsu(stg); // 加槓の場合は槍槓リーチ一発があるので一旦スルー
    }

    let s = event.seat;
    stg.turn = s;
    let pl = &mut stg.players[s];
    if event.is_pao {
        pl.pao = Some(stg.turn);
    }

    let mut idx; // Vec<Meld>のインデックス
    match event.meld_type {
        MeldType::Chi | MeldType::Pon | MeldType::Minkan => {
            let lt = stg.last_tile.unwrap();
            assert!(lt.1 == ActionType::Discard);

            pl.is_menzen = false;
            if event.meld_type == MeldType::Minkan {
                pl.is_rinshan = true;
            }
            idx = pl.melds.len();
            let mut tiles = event.consumed.clone();
            let mut froms = vec![s; tiles.len()];
            tiles.push(lt.2);
            froms.push(lt.0);
            let m = Meld {
                step: stg.step,
                seat: s,
                meld_type: event.meld_type,
                tiles,
                froms,
            };
            let &(prev_s, prev_i) = stg.discards.last().unwrap();
            stg.players[prev_s].discards[prev_i].meld = Some((s, idx));
            stg.players[s].melds.push(m);
            stg.players[prev_s].is_nagashimangan = false;
        }
        MeldType::Ankan => {
            pl.is_rinshan = true;
            idx = pl.melds.len();
            let tiles = event.consumed.clone();
            let froms = vec![s; tiles.len()];
            let m = Meld {
                step: stg.step,
                seat: s,
                meld_type: MeldType::Ankan,
                tiles,
                froms,
            };
            pl.melds.push(m);

            let t = event.consumed[0];
            stg.last_tile = Some((s, ActionType::Ankan, t)); // 国士の暗槓ロン
        }
        MeldType::Kakan => {
            pl.is_rinshan = true;
            idx = 0;
            let t = event.consumed[0];
            for m in &mut pl.melds {
                if m.tiles[0] == t || m.tiles[1] == t {
                    m.step = stg.step;
                    m.meld_type = MeldType::Kakan;
                    m.tiles.push(t);
                    m.froms.push(s);
                    break;
                }
                idx += 1;
            }

            stg.last_tile = Some((s, ActionType::Kakan, t)); // 槍槓+フリテン用
        }
    }

    for &t in &event.consumed {
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

fn event_kita(stg: &mut Stage, event: &EventKita) {
    let s = event.seat;
    let t = Tile(TZ, WN); // z4
    let pl = &mut stg.players[s];
    let idx = pl.kitas.len();
    let k = Nukidora {
        step: stg.step,
        seat: s,
        is_drop: event.is_drop,
    };

    if pl.is_shown {
        player_dec_tile(pl, t);
        table_edit(stg, t, H(s), K(s, idx));
    } else {
        player_dec_tile(pl, Z8);
        table_edit(stg, t, U, K(s, idx));
    }

    stg.players[s].kitas.push(k);
    stg.last_tile = Some((s, ActionType::Nukidora, t));
}

fn event_dora(stg: &mut Stage, event: &EventDora) {
    table_edit(stg, event.tile, TileState::U, TileState::R);
    stg.doras.push(event.tile);
}

fn event_win(stg: &mut Stage, event: &EventWin) {
    for ctx in &event.contexts {
        update_scores(stg, &ctx.delta_scores);
    }
}

fn event_draw(stg: &mut Stage, event: &EventDraw) {
    update_scores(stg, &event.delta_scores);
}

fn event_end(_stg: &mut Stage, _event: &EventEnd) {}

// [Utility]
fn table_edit(stg: &mut Stage, tile: Tile, old: TileState, new: TileState) {
    let tn = tile.to_normal();
    let te = &mut stg.tile_states[tn.0][tn.1];
    // println!("[table_edit] {}: {:?} | {:?} => {:?}", tile, te, old, new);
    let i = te.iter().position(|&x| x == old).unwrap();
    te[i] = new;
    te.sort();
}

fn disable_ippatsu(stg: &mut Stage) {
    for s in 0..SEAT {
        stg.players[s].is_ippatsu = false;
    }
}

fn update_after_discard_completed(stg: &mut Stage) {
    // 他のプレイヤーの捨て牌,または加槓した牌の見逃しフリテン
    if let Some((s, tp, t)) = stg.last_tile {
        match tp {
            ActionType::Discard | ActionType::Kakan => {
                for s2 in 0..SEAT {
                    let pl = &mut stg.players[s2];
                    if pl.winning_tiles.contains(&t) {
                        if s2 == s || pl.is_riichi {
                            pl.is_furiten = true; // 自分で和了牌を捨てた場合と見逃しした場合はフリテン
                        } else {
                            pl.is_furiten_other = true;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // リーチがロンされずに成立した場合の供託への点棒追加
    if let Some(s) = stg.last_riichi {
        stg.players[s].score -= 1000;
        stg.riichi_sticks += 1;
        stg.last_riichi = None;
    }
}

fn update_scores(stg: &mut Stage, points: &[Point; SEAT]) {
    for s in 0..SEAT {
        stg.players[s].score += points[s];
    }

    let scores = stg.players.iter().map(|pl| pl.score).collect();
    let ranks = rank_by_rank_vec(&scores);
    for s in 0..SEAT {
        stg.players[s].rank = ranks[s];
    }
}

#[inline]
fn player_inc_tile(pl: &mut Player, tile: Tile) {
    inc_tile(&mut pl.hand, tile);
}

#[inline]
fn player_dec_tile(pl: &mut Player, tile: Tile) {
    dec_tile(&mut pl.hand, tile);
}

fn get_winning_tiles(pl: &Player) -> Vec<Tile> {
    let wts0 = calc_tiles_to_kokushimusou_win(&pl.hand);
    let wts1 = calc_tiles_to_normal_win(&pl.hand);
    let wts2 = calc_tiles_to_chiitoitsu_win(&pl.hand);
    let mut wts = [wts0, wts1, wts2].concat();
    wts.sort();
    wts.dedup();
    wts
}
