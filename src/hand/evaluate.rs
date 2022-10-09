use super::parse::*;
use super::point::*;
use super::win::*;
use super::yaku::*;
use crate::model::*;
use crate::tool::common::*;

pub fn evaluate_hand_tsumo(stg: &Stage, ura_dora_wall: &Vec<Tile>) -> Option<ScoreContext> {
    let pl = &stg.players[stg.turn];
    if !pl.is_shown {
        return None;
    }

    if !pl.winning_tiles.contains(&pl.drawn.unwrap().to_normal()) {
        return None;
    }

    let mut yf = YakuFlags {
        menzentsumo: true,
        riichi: pl.is_riichi && !pl.is_daburii,
        dabururiichi: pl.is_daburii,
        ippatsu: pl.is_ippatsu,
        haiteiraoyue: stg.wall_count == 0,
        rinshankaihou: pl.is_rinshan,
        tenhou: false, // TODO
        tiihou: false, // TODO
        ..Default::default()
    };
    for m in &pl.melds {
        if m.meld_type != MeldType::Ankan {
            yf.menzentsumo = false;
        }
    }
    if check_tenhou_tiihou(stg, stg.turn) {
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

    if let Some(res) = evaluate_hand(
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
    ) {
        if !res.yakus.is_empty() {
            return Some(res);
        }
    }

    None
}

pub fn evaluate_hand_ron(
    stg: &Stage,
    ura_dora_wall: &Vec<Tile>,
    seat: Seat,
) -> Option<ScoreContext> {
    if seat == stg.turn {
        return None;
    }

    let pl = &stg.players[seat];
    if let Some((_, _, t)) = stg.last_tile {
        if !pl.winning_tiles.contains(&t.to_normal()) {
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
    if check_tenhou_tiihou(stg, stg.turn) {
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

    if let Some(res) = evaluate_hand(
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
// この関数は本場数の得点を計算しない.
pub fn evaluate_hand(
    hand: &TileTable,       // 手牌(鳴き以外, ロンの場合でも和了牌を含む)
    melds: &Vec<Meld>,      // 鳴き
    doras: &Vec<Tile>,      // ドラ表示牌 (注:ドラそのものではない)
    ura_doras: &Vec<Tile>,  // 裏ドラ表示牌 リーチしていない場合は空
    winning_tile: Tile,     // 上がり牌
    is_drawn: bool,         // ツモ和了
    is_dealer: bool,        // 親番
    prevalent_wind: Index,  // 場風 (東: 1, 南: 2, 西: 3, 北: 4)
    seat_wind: Index,       // 自風 (同上)
    yaku_flags: &YakuFlags, // 和了形だった場合に自動的に付与される役(特殊条件役)のフラグ
) -> Option<ScoreContext> {
    let mut phs = vec![];
    phs.append(&mut parse_into_normal_win(hand));
    if melds.is_empty() {
        phs.append(&mut parse_into_chiitoitsu_win(hand));
        phs.append(&mut parse_into_kokusimusou_win(hand));
    }

    let mut wins = vec![]; // 和了形のリスト (無役を含む)
    let pm = parse_melds(melds);
    for mut ph in phs.into_iter() {
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

    let mut results = vec![];
    for ctx in wins {
        let fu = ctx.calc_fu();
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
        let points = get_points(is_dealer, fu, fan, yakuman);
        let title = get_score_title(fu, fan, yakuman);
        let score = if is_drawn {
            if is_dealer {
                points.1 * 3
            } else {
                points.1 * 2 + points.2
            }
        } else {
            points.0
        };
        results.push(ScoreContext {
            yakus,
            fu,
            fan,
            yakuman,
            score,
            points,
            title,
        });
    }

    // 和了形に複数の解釈が可能な場合,最も得点の高いものを採用
    results.sort_by_key(|r| r.points.0);
    results.pop()
}

pub struct WinningTile {
    tile: Tile,     // 和了牌
    has_yaku: bool, // 出和了可能な役があるかどうか
}

pub struct Tenpai {
    discard_tile: Tile,              // 聴牌になる打牌
    winning_tiles: Vec<WinningTile>, // 聴牌になる和了牌のリスト
    is_furiten: bool,                // フリテンの有無
}

// 聴牌になる打牌を各々の上がり牌に対するスコア(翻数)やフリテンの情報を添えて返却
// 返り値: [{打牌, [{和了牌, 役の有無, フリテンの有無}]}]
pub fn evaluate_hand_tenpai_discards(
    hand: &TileTable,
    melds: &Vec<Meld>,
    prevalent_wind: Index,
    seat_wind: Index,
    discards: &Vec<Discard>,
) -> Vec<Tenpai> {
    let mut comb: Vec<(Tile, Tile)> = vec![]; // (打牌, 和了牌)の組み合わせ
    for (d, wts) in calc_discards_to_normal_tenpai(hand).into_iter() {
        for wt in wts.into_iter() {
            comb.push((d, wt));
        }
    }
    for (d, wts) in calc_discards_to_chiitoitsu_tenpai(hand).into_iter() {
        for wt in wts.into_iter() {
            comb.push((d, wt));
        }
    }
    for (d, wts) in calc_discards_to_kokushimusou_tenpai(hand).into_iter() {
        for wt in wts.into_iter() {
            comb.push((d, wt));
        }
    }
    comb.sort();
    comb.dedup();

    let yf = YakuFlags::default();
    let mut res: Vec<Tenpai> = vec![];
    for (d, wt) in comb {
        if res.is_empty() || res.last().unwrap().discard_tile != d {
            res.push(Tenpai {
                discard_tile: d,
                winning_tiles: vec![],
                is_furiten: false,
            })
        }

        let mut h = *hand;
        dec_tile(&mut h, d);
        inc_tile(&mut h, wt);
        let sc = evaluate_hand(
            &h,
            melds,
            &vec![],
            &vec![],
            wt,
            false,
            false,
            prevalent_wind,
            seat_wind,
            &yf,
        );
        let wt_info = WinningTile {
            tile: wt,
            has_yaku: sc.is_some(),
        };

        let tenpai = res.last_mut().unwrap();
        tenpai.winning_tiles.push(wt_info);
        if !tenpai.is_furiten {
            for d2 in discards {
                if d2.tile.to_normal() == wt.to_normal() {
                    tenpai.is_furiten = true;
                }
            }
        }
    }

    res
}

fn check_tenhou_tiihou(stg: &Stage, seat: Seat) -> bool {
    if !stg.players[seat].discards.is_empty() {
        return false;
    } else {
        for s in 0..SEAT {
            if !stg.players[s].melds.is_empty() {
                return false;
            }
        }
    }
    true
}

#[test]
fn test_tenpai() {}
