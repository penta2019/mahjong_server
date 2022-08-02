use super::parse::*;
use super::point::*;
use super::yaku::*;
use crate::model::*;

pub fn evaluate_hand_tsumo(stg: &Stage, ura_dora_wall: &Vec<Tile>) -> Option<WinContext> {
    let pl = &stg.players[stg.turn];
    if !pl.is_shown {
        return None;
    }

    if !pl.win_tiles.contains(&pl.drawn.unwrap().to_normal()) {
        return None;
    }

    let mut yf = YakuFlags {
        menzentsumo: true,
        riichi: pl.is_riichi && !pl.is_daburii,
        dabururiichi: pl.is_daburii,
        ippatsu: pl.is_ippatsu,
        haiteiraoyue: stg.left_tile_count == 0,
        rinshankaihou: pl.is_rinshan,
        tenhou: false, // TODO
        tiihou: false, // TODO
        ..Default::default()
    };
    for m in &pl.melds {
        if m.type_ != MeldType::Ankan {
            yf.menzentsumo = false;
        }
    }

    let ura_doras = if !ura_dora_wall.is_empty() && pl.is_riichi {
        ura_dora_wall[0..stg.doras.len()].to_vec()
    } else {
        vec![]
    };

    if let Some(res) = evaluate_hand(
        &pl.hand,
        &pl.melds,
        &stg.doras,
        &ura_doras,
        pl.drawn.unwrap(),
        true,
        stg.is_dealer(pl.seat),
        stg.get_prevalent_wind(),
        stg.get_seat_wind(pl.seat),
        &yf,
    ) {
        if !res.yakus.is_empty() {
            return Some(res);
        }
    }

    None
}

pub fn evaluate_hand_ron(stg: &Stage, ura_dora_wall: &Vec<Tile>, seat: Seat) -> Option<WinContext> {
    if seat == stg.turn {
        return None;
    }

    let pl = &stg.players[seat];
    if let Some((_, _, t)) = stg.last_tile {
        if !pl.win_tiles.contains(&t.to_normal()) {
            return None;
        }
    }
    if !pl.is_shown || pl.is_furiten || pl.is_furiten_other {
        return None;
    }

    let (tp, t) = if let Some((_, tp, t)) = stg.last_tile {
        (tp, t)
    } else {
        return None;
    };

    let mut hand = pl.hand;
    if t.1 == 0 {
        // 赤5
        hand[t.0][0] += 1;
        hand[t.0][5] += 1;
    } else {
        hand[t.0][t.1] += 1;
    }

    let mut yf = YakuFlags {
        riichi: pl.is_riichi && !pl.is_daburii,
        dabururiichi: pl.is_daburii,
        ippatsu: pl.is_ippatsu,
        ..Default::default()
    };
    match tp {
        ActionType::Discard => yf.houteiraoyui = stg.left_tile_count == 0,
        ActionType::Kakan => yf.chankan = true,
        ActionType::Ankan => {
            if parse_into_kokusimusou_win(&hand).is_empty() {
                return None; // 暗槓のロンは国士無双のみ
            }
        }
        _ => panic!(),
    }

    let ura_doras = if !ura_dora_wall.is_empty() && pl.is_riichi {
        ura_dora_wall[0..stg.doras.len()].to_vec()
    } else {
        vec![]
    };

    if let Some(res) = evaluate_hand(
        &hand,
        &pl.melds,
        &stg.doras,
        &ura_doras,
        t,
        false,
        stg.is_dealer(pl.seat),
        stg.get_prevalent_wind(),
        stg.get_seat_wind(pl.seat),
        &yf,
    ) {
        if !res.yakus.is_empty() {
            return Some(res);
        }
    }

    None
}

// 和了形である場合,最も高得点となるような役の組み合わせのSome(Result)を返却
// 和了形でない場合,Noneを返却
// 和了形でも無役の場合はResultの中身がyaku: [], points(0, 0, 0)となる.
pub fn evaluate_hand(
    hand: &TileTable,       // 手牌(鳴き以外)
    melds: &Vec<Meld>,      // 鳴き
    doras: &Vec<Tile>,      // ドラ表示牌 (注:ドラそのものではない)
    ura_doras: &Vec<Tile>,  // 裏ドラ表示牌 リーチしていない場合は空
    win_tile: Tile,         // 上がり牌
    is_drawn: bool,         // ツモ和了
    is_dealer: bool,        // 親番
    prevalent_wind: Index,  // 場風 (東: 1, 南: 2, 西: 3, 北: 4)
    seat_wind: Index,       // 自風 (同上)
    yaku_flags: &YakuFlags, // 和了形だった場合に自動的に付与される役(特殊条件役)のフラグ
) -> Option<WinContext> {
    let mut wins = vec![]; // 和了形のリスト (無役を含む)

    // 和了(通常)
    let pm = parse_melds(melds);
    for mut ph in parse_into_normal_win(hand).into_iter() {
        ph.append(&mut pm.clone());
        let ctx = YakuContext::new(
            *hand,
            ph,
            win_tile,
            prevalent_wind,
            seat_wind,
            is_drawn,
            yaku_flags.clone(),
        );
        wins.push(ctx);
    }

    // 和了(七対子)
    for ph in parse_into_chiitoitsu_win(hand).into_iter() {
        let ctx = YakuContext::new(
            *hand,
            ph,
            win_tile,
            prevalent_wind,
            seat_wind,
            is_drawn,
            yaku_flags.clone(),
        );
        wins.push(ctx);
    }

    // 和了(国士無双)
    for ph in parse_into_kokusimusou_win(hand).into_iter() {
        let ctx = YakuContext::new(
            *hand,
            ph,
            win_tile,
            prevalent_wind,
            seat_wind,
            is_drawn,
            yaku_flags.clone(),
        );
        wins.push(ctx);
    }

    if wins.is_empty() {
        return None; // 和了形以外
    }

    let n_dora = count_dora(hand, melds, doras);
    let mut n_red_dora = hand[0][0] + hand[1][0] + hand[2][0];
    for m in melds {
        for t in &m.tiles {
            if t.1 == 0 {
                n_red_dora += 1;
            }
        }
    }
    let n_ura_dora = if yaku_flags.riichi || yaku_flags.dabururiichi {
        count_dora(hand, melds, ura_doras)
    } else {
        0
    };

    let mut results = vec![];
    for ctx in wins {
        let hand = tiles_from_tile_table(hand);
        let fu = ctx.calc_fu();
        let (yakus, mut fan, yakuman_times) = ctx.calc_yaku();
        if yakus.is_empty() {
            continue; // 無役
        }
        let mut yakus: Vec<(String, usize)> = yakus
            .iter()
            .map(|y| {
                let fan = if ctx.is_open() {
                    y.fan_open
                } else {
                    y.fan_close
                };
                (y.name.to_string(), fan)
            })
            .collect();
        if yakuman_times == 0 {
            fan += n_dora + n_red_dora + n_ura_dora;
            if n_dora != 0 {
                yakus.push(("ドラ".to_string(), n_dora));
            }
            if n_red_dora != 0 {
                yakus.push(("赤ドラ".to_string(), n_red_dora));
            }
            if n_ura_dora != 0 {
                yakus.push(("裏ドラ".to_string(), n_ura_dora));
            }
        }
        let points = get_points(is_dealer, fu, fan, yakuman_times);
        let score_title = get_score_title(fu, fan, yakuman_times);
        results.push(WinContext {
            hand,
            yakus,
            fu,
            fan,
            yakuman_times,
            score_title,
            points,
        });
    }

    results.sort_by_key(|r| r.points.0);
    results.pop()
}

// ドラ表示牌のリストを受け取ってドラ評価値のテーブルを返却
fn create_dora_table(doras: &Vec<Tile>) -> TileTable {
    let mut dt = TileTable::default();
    for d in doras {
        let ni = if d.is_hornor() {
            match d.1 {
                WN => WE,
                DR => DW,
                i => i + 1,
            }
        } else {
            match d.1 {
                9 => 1,
                0 => 6,
                _ => d.1 + 1,
            }
        };
        dt[d.0][ni] += 1;
    }

    dt
}

// ドラ(赤5を含む)の数を勘定
fn count_dora(hand: &TileTable, melds: &Vec<Meld>, doras: &Vec<Tile>) -> usize {
    let dt = create_dora_table(doras);
    let mut n_dora = 0;

    for ti in 0..TYPE {
        for ni in 1..TNUM {
            n_dora += dt[ti][ni] * hand[ti][ni];
        }
    }

    for m in melds {
        for t in &m.tiles {
            let t = t.to_normal();
            n_dora += dt[t.0][t.1];
        }
    }

    n_dora
}
