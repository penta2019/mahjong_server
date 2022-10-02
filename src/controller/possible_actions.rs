use crate::hand::*;
use crate::model::*;

// [Turn Action Check]
// プレイヤーのツモ番に可能な操作をチェックする
// fn(&Stage) -> Option<Action>

pub fn calc_possible_turn_actions(stg: &Stage, melding: &Option<Action>) -> Vec<Action> {
    if let Some(act) = melding {
        match act.action_type {
            ActionType::Chi | ActionType::Pon => {
                // チー,ポンのあとは打牌のみ
                return vec![Action::new(
                    ActionType::Discard,
                    calc_restricted_discards(act),
                )];
            }
            _ => {}
        }
    }

    let mut acts = vec![Action::nop()];
    if !stg.players[stg.turn].is_riichi {
        acts.push(Action::new(ActionType::Discard, vec![]));
    }
    acts.append(&mut check_ankan(stg));
    acts.append(&mut check_kakan(stg));
    acts.append(&mut check_riichi(stg));
    acts.append(&mut check_tsumo(stg));
    acts.append(&mut check_kyushukyuhai(stg));
    acts.append(&mut check_kita(stg));
    acts
}

fn check_riichi(stg: &Stage) -> Vec<Action> {
    if stg.wall_count < 4 {
        return vec![];
    }

    let pl = &stg.players[stg.turn];
    if pl.is_riichi || !pl.is_menzen || pl.score < 1000 {
        return vec![];
    }

    let ds1 = calc_discards_to_normal_tenpai(&pl.hand);
    let ds2 = calc_discards_to_chiitoitsu_tenpai(&pl.hand);
    let ds3 = calc_discards_to_kokushimusou_tenpai(&pl.hand);
    let mut tiles = vec![];
    for ds in [ds1, ds2, ds3] {
        for (d, _) in ds {
            tiles.push(d);
        }
    }
    tiles.sort();
    tiles.dedup();

    if tiles.is_empty() {
        vec![]
    } else {
        vec![Action::new(ActionType::Riichi, tiles)]
    }
}

fn check_tsumo(stg: &Stage) -> Vec<Action> {
    if evaluate_hand_tsumo(stg, &vec![]).is_some() {
        vec![Action::tsumo()]
    } else {
        vec![]
    }
}

fn check_ankan(stg: &Stage) -> Vec<Action> {
    if stg.wall_count == 0 || stg.doras.len() == 5 {
        return vec![];
    }

    let ankan = |t: Tile| {
        if t.is_suit() && t.1 == 5 {
            // 赤5を含む暗槓
            Action::ankan(vec![Tile(t.0, 0), t, t, t])
        } else {
            Action::ankan(vec![t, t, t, t])
        }
    };

    let pl = &stg.players[stg.turn];
    let mut acts = vec![];
    if pl.is_riichi {
        // リーチ中でも待ちが変わらない暗槓は可能
        if let Some(t) = pl.drawn {
            let t = t.to_normal();
            if pl.hand[t.0][t.1] == 4 {
                let mut h = pl.hand;

                h[t.0][t.1] -= 1;
                let mut v1 = calc_tiles_to_normal_win(&h);
                v1.sort();

                h[t.0][t.1] -= 3;
                let mut v2 = calc_tiles_to_normal_win(&h);
                v2.sort();

                if v1 == v2 {
                    acts.push(ankan(t));
                }
            }
        }
    } else {
        for ti in 0..TYPE {
            for ni in 1..TNUM {
                if pl.hand[ti][ni] == 4 {
                    acts.push(ankan(Tile(ti, ni)));
                }
            }
        }
    }

    acts
}

fn check_kakan(stg: &Stage) -> Vec<Action> {
    if stg.wall_count == 0 || stg.doras.len() == 5 {
        return vec![];
    }

    let pl = &stg.players[stg.turn];
    if pl.is_riichi {
        return vec![];
    }

    let mut acts = vec![];
    for m in &pl.melds {
        if m.meld_type == MeldType::Pon {
            let t = m.tiles[0].to_normal();
            if pl.hand[t.0][t.1] != 0 {
                acts.push(if t.is_suit() && t.1 == 5 && pl.hand[t.0][0] > 0 {
                    Action::kakan(Tile(t.0, 0)) // 赤5
                } else {
                    Action::kakan(t)
                });
            }
        }
    }

    acts
}

fn check_kyushukyuhai(stg: &Stage) -> Vec<Action> {
    let pl = &stg.players[stg.turn];
    if !pl.discards.is_empty() {
        return vec![];
    }

    for pl2 in &stg.players {
        if !pl2.melds.is_empty() {
            return vec![];
        }
    }

    let mut c = 0;
    for ti in 0..TZ {
        if pl.hand[ti][1] != 0 {
            c += 1;
        }
        if pl.hand[ti][9] != 0 {
            c += 1;
        }
    }
    for ni in 1..8 {
        if pl.hand[TZ][ni] != 0 {
            c += 1;
        }
    }
    if c < 9 {
        return vec![];
    }

    vec![Action::kyushukyuhai()]
}

fn check_kita(stg: &Stage) -> Vec<Action> {
    if !stg.is_sanma {
        return vec![];
    }

    //　海底不可
    if stg.wall_count == 0 {
        return vec![];
    }

    let mut acts = vec![];
    if stg.players[stg.turn].hand[TZ][WN] != 0 {
        acts.push(Action::kita());
    }

    acts
}

// [Call Action Check]
// ツモ番のプレイヤーが打牌を行ったあとに,他のプレイヤーが可能な操作をチェックする
// fn(&Stage) -> Vec<(Seat, Action)>
// ロン以外の返り値のリストは要素が2つ以上になることはないが一貫性のためVecを返却する

pub fn calc_possible_call_actions(stg: &Stage, can_meld: bool) -> [Vec<Action>; SEAT] {
    let mut acts_list: [Vec<Action>; SEAT] = Default::default();
    for s in 0..SEAT {
        acts_list[s].push(Action::nop());
    }
    // 暗槓,加槓,四槓散了に対して他家はロン以外の操作は行えない
    if can_meld {
        for (s, act) in check_chi(stg) {
            acts_list[s].push(act);
        }
        for (s, act) in check_pon(stg) {
            acts_list[s].push(act);
        }
        for (s, act) in check_minkan(stg) {
            acts_list[s].push(act);
        }
    }
    for (s, act) in check_ron(stg) {
        acts_list[s].push(act);
    }
    acts_list
}

fn check_chi(stg: &Stage) -> Vec<(Seat, Action)> {
    if stg.wall_count == 0 {
        return vec![];
    }

    let pl_turn = &stg.players[stg.turn];
    let d = pl_turn.discards.last().unwrap().tile.to_normal();
    if d.is_hornor() {
        return vec![];
    }

    let s = (stg.turn + 1) % SEAT;
    if stg.players[s].is_riichi {
        return vec![];
    }

    let mut check: Vec<(Tnum, Tnum)> = vec![];
    let Tile(ti, ni) = d;

    if 3 <= ni {
        check.push((ni - 2, ni - 1)); // 右端をチー
    }
    if (2..=8).contains(&ni) {
        check.push((ni - 1, ni + 1)); // 嵌張をチー
    }
    if ni <= 7 {
        check.push((ni + 1, ni + 2)); // 左端をチー
    }

    let h = &stg.players[s].hand;
    let mut acts = vec![];
    for (ni0, ni1) in check {
        for t0 in tiles_with_red5(h, Tile(ti, ni0)) {
            for t1 in tiles_with_red5(h, Tile(ti, ni1)) {
                acts.push((s, Action::chi(vec![t0, t1])));
            }
        }
    }

    acts
}

fn check_pon(stg: &Stage) -> Vec<(Seat, Action)> {
    if stg.wall_count == 0 {
        return vec![];
    }

    let d = stg.last_tile.unwrap().2;
    let t = d.to_normal();
    let mut acts = vec![];
    for s in 0..SEAT {
        let pl = &stg.players[s];
        if pl.hand[t.0][t.1] < 2 || stg.turn == s || pl.is_riichi {
            continue;
        }

        let t0 = Tile(t.0, 0);
        let pon = Action::pon(vec![t, t]);
        let pon0 = Action::pon(vec![t0, t]); // 手牌の赤5を含むPon
        if t.is_suit() && t.1 == 5 && pl.hand[t.0][0] > 0 {
            // 赤5がある場合
            if pl.hand[t.0][t.1] > 2 {
                acts.push((s, pon));
                acts.push((s, pon0));
            } else {
                acts.push((s, pon0));
            }
        } else {
            // 5以外または赤なし
            acts.push((s, pon));
        }
    }
    acts
}

fn check_minkan(stg: &Stage) -> Vec<(Seat, Action)> {
    if stg.wall_count == 0 || stg.doras.len() == 5 {
        return vec![];
    }

    let d = stg.last_tile.unwrap().2;
    let t = d.to_normal();
    let mut acts = vec![];
    for s in 0..SEAT {
        let pl = &stg.players[s];
        if pl.hand[t.0][t.1] != 3 || stg.turn == s || pl.is_riichi {
            continue;
        }

        let cs = if t.is_suit() && t.1 == 5 && pl.hand[t.0][0] > 0 {
            Action::minkan(vec![Tile(t.0, 0), t, t])
        } else {
            Action::minkan(vec![t, t, t])
        };
        acts.push((s, cs));
    }
    acts
}

fn check_ron(stg: &Stage) -> Vec<(Seat, Action)> {
    let mut acts = vec![];
    for s in 0..SEAT {
        if evaluate_hand_ron(stg, &vec![], s).is_some() {
            acts.push((s, Action::ron()));
        }
    }
    acts
}

// 鳴き後の組み換え禁止の牌
fn calc_restricted_discards(act: &Action) -> Vec<Tile> {
    let mut v = vec![];
    let tp = act.action_type;
    let cs = &act.tiles;
    match tp {
        ActionType::Chi => {
            // 赤5が混じっている可能性を考慮
            let (t0, t1) = (cs[0].to_normal(), cs[1].to_normal());
            let ti = t0.0;
            let ni0 = t0.1;
            let ni1 = t1.1;
            let s = std::cmp::min(ni0, ni1);
            let b = std::cmp::max(ni0, ni1);
            if s + 1 == b {
                // リャンメン・ペンチャン
                let i = s - 1;
                if 0 < i {
                    v.push(Tile(ti, i));
                }
                let i = b + 1;
                if i < TNUM {
                    v.push(Tile(ti, i))
                }
            } else {
                // カンチャン
                let i = s + 1;
                v.push(Tile(ti, i));
            }
        }
        ActionType::Pon => {
            v.push(cs[0].to_normal());
        }
        _ => return vec![],
    }

    // 組み換え禁止の牌のなかに5がある場合, 赤5も追加
    for t in &v {
        if t.is_suit() && t.1 == 5 {
            v.push(Tile(v[0].0, 0));
            break;
        }
    }

    v
}
