use super::{parse::*, point::*, yaku::*};
use crate::{control::common::*, model::*};

pub fn evaluate_hand_tsumo(stg: &Stage, ura_dora_wall: &[Tile]) -> Option<ScoreContext> {
    let pl = &stg.players[stg.turn];
    if !pl.is_shown {
        return None;
    }

    if !pl.winning_tiles.contains(&pl.drawn.unwrap().to_normal()) {
        return None;
    }

    let mut yf = YakuFlags {
        menzentsumo: pl.melds.iter().all(|m| m.meld_type == MeldType::Ankan),
        riichi: pl.is_riichi && !pl.is_daburii,
        dabururiichi: pl.is_daburii,
        ippatsu: pl.is_ippatsu,
        haiteiraoyue: stg.wall_count == 0,
        rinshankaihou: pl.is_rinshan,
        tenhou: false,
        tiihou: false,
        ..Default::default()
    };
    if is_no_meld_turn1(stg, stg.turn) {
        if is_dealer(stg, stg.turn) {
            yf.tenhou = true;
        } else {
            yf.tiihou = true;
        }
    }

    let ura_doras = if !ura_dora_wall.is_empty() && pl.is_riichi {
        ura_dora_wall[0..stg.doras.len()].to_vec()
    } else {
        vec![]
    };

    if let Some((ctx, _)) = evaluate_hand(
        &pl.hand,
        &pl.melds,
        &stg.doras,
        &ura_doras,
        pl.drawn.unwrap(),
        true,
        is_dealer(stg, pl.seat),
        get_prevalent_wind(stg),
        get_seat_wind(stg, pl.seat),
        &yf,
    ) && !ctx.yakus.is_empty()
    {
        return Some(ctx);
    }

    None
}

pub fn evaluate_hand_ron(stg: &Stage, ura_dora_wall: &[Tile], seat: Seat) -> Option<ScoreContext> {
    if seat == stg.turn {
        return None;
    }

    let pl = &stg.players[seat];
    if let Some((_, _, t)) = stg.last_tile
        && !pl.winning_tiles.contains(&t.to_normal())
    {
        return None;
    }
    if !pl.is_shown || pl.is_furiten || pl.is_furiten_other {
        return None;
    }

    let (tp, t) = if let Some((_, tp, t)) = stg.last_tile {
        (tp, t)
    } else {
        return None;
    };

    // 和了牌を追加
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
        ActionType::Discard => yf.houteiraoyui = stg.wall_count == 0,
        ActionType::Kakan => yf.chankan = true,
        ActionType::Ankan => {
            if parse_into_kokusimusou_win(&hand).is_empty() {
                return None; // 暗槓のロンは国士無双のみ
            }
        }
        _ => panic!(),
    }
    if is_no_meld_turn1(stg, stg.turn) {
        if is_dealer(stg, stg.turn) {
            yf.tenhou = true;
        } else {
            yf.tiihou = true;
        }
    }

    let ura_doras = if !ura_dora_wall.is_empty() && pl.is_riichi {
        ura_dora_wall[0..stg.doras.len()].to_vec()
    } else {
        vec![]
    };

    if let Some((ctx, _)) = evaluate_hand(
        &hand,
        &pl.melds,
        &stg.doras,
        &ura_doras,
        t,
        false,
        is_dealer(stg, pl.seat),
        get_prevalent_wind(stg),
        get_seat_wind(stg, pl.seat),
        &yf,
    ) && !ctx.yakus.is_empty()
    {
        return Some(ctx);
    }

    None
}

// 和了形である場合,最も高得点となるような役の組み合わせのSome(Result)を返却
// 和了形でない場合,Noneを返却
// 和了形でも無役の場合はResultの中身がyaku: [], points(0, 0, 0)となる.
// この関数は本場数の得点を計算しない.
pub fn evaluate_hand(
    hand: &TileTable,       // 手牌(鳴き以外, ロンの場合でも和了牌を含む)
    melds: &[Meld],         // 鳴き
    doras: &[Tile],         // ドラ表示牌 (注:ドラそのものではない)
    ura_doras: &[Tile],     // 裏ドラ表示牌 リーチしていない場合は空
    winning_tile: Tile,     // 上がり牌
    is_drawn: bool,         // ツモ和了
    is_dealer: bool,        // 親番
    prevalent_wind: Index,  // 場風 (東: 1, 南: 2, 西: 3, 北: 4)
    seat_wind: Index,       // 自風 (同上)
    yaku_flags: &YakuFlags, // 和了形だった場合に自動的に付与される役(特殊条件役)のフラグ
) -> Option<(ScoreContext, Vec<Fu>)> {
    let mut phs = vec![];
    phs.append(&mut parse_into_normal_win(hand));
    if melds.is_empty() {
        phs.append(&mut parse_into_chiitoitsu_win(hand));
        phs.append(&mut parse_into_kokusimusou_win(hand));
    }

    let mut wins = vec![]; // 和了形のリスト (無役を含む)
    let pm = parse_melds(melds);
    for mut ph in phs {
        ph.append(&mut pm.clone());
        match ph.len() {
            0 | 5 | 7 => {} // 国士, 通常, 七対子
            _ => continue,  // 無効な和了形
        }
        wins.push(YakuContext::new(
            *hand,
            ph,
            winning_tile,
            prevalent_wind,
            seat_wind,
            is_drawn,
            yaku_flags.clone(),
        ));
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

    let mut ctxs = vec![];
    for ctx in wins {
        let fus = ctx.calc_fu();
        let mut fu = fus.iter().map(|(fu, _)| fu).sum::<usize>(); // 符の総和
        if fu != 25 {
            fu = fu.div_ceil(10) * 10; // 七対子以外なら１の位は切り上げ
        }

        let (yakus, mut fan, yakuman) = ctx.calc_yaku();
        if yakus.is_empty() {
            continue; // 無役
        }
        let mut yakus: Vec<Yaku> = yakus
            .iter()
            .map(|y| {
                let fan = if y.yakuman > 0 {
                    y.yakuman
                } else {
                    if ctx.is_open() {
                        y.fan_open
                    } else {
                        y.fan_close
                    }
                };
                Yaku {
                    name: y.name.to_string(),
                    fan,
                }
            })
            .collect();
        if yakuman == 0 {
            fan += n_dora + n_red_dora + n_ura_dora;
            if n_dora != 0 {
                yakus.push(Yaku {
                    name: "ドラ".to_string(),
                    fan: n_dora,
                });
            }
            if n_red_dora != 0 {
                yakus.push(Yaku {
                    name: "赤ドラ".to_string(),
                    fan: n_red_dora,
                });
            }
            if n_ura_dora != 0 {
                yakus.push(Yaku {
                    name: "裏ドラ".to_string(),
                    fan: n_ura_dora,
                });
            }
        }

        let (points, title) = calc_points(is_dealer, fu, fan, yakuman);
        let score = if is_drawn {
            if is_dealer {
                points.1 * 3
            } else {
                points.1 * 2 + points.2
            }
        } else {
            points.0
        };
        ctxs.push((
            ScoreContext {
                yakus,
                fu,
                fan,
                yakuman,
                score,
                points,
                title,
            },
            fus,
        ));
    }

    // 和了形に複数の解釈が可能な場合,最も得点の高いものを採用
    ctxs.sort_by_key(|(ctx, _)| (ctx.points.0, ctx.fan, ctx.fu));
    ctxs.pop()
}
