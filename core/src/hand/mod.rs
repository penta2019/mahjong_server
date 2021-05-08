pub mod score;
pub mod split;
pub mod win;
pub mod yaku;

use crate::model::*;

use score::*;
use split::*;
use yaku::*;

#[derive(Debug)]
pub struct WinContext {
    pub fu: i32,                  // 符数
    pub yaku: Vec<&'static Yaku>, // 役一覧(ドラは含まない)
    pub n_dora: usize,            // ドラの数(裏ドラは含まない)
    pub n_ura_dora: usize,        // 裏ドラの数
    pub is_yakuman: bool,         // 役満かどうか
    pub fan_mag: usize,           // 翻数(ドラを含む), 役満倍率(is_yakuman=trueの時)
    pub pay_scores: PayScores,    // 支払い得点
}

pub fn evaluate_hand_tsumo(stg: &Stage, ura_dora_wall: &Vec<Tile>) -> Option<WinContext> {
    let pl = &stg.players[stg.turn];
    if !pl.is_shown {
        return None;
    }

    let mut yf = YakuFlags::default();
    yf.menzentsumo = pl.melds.is_empty();
    yf.riichi = pl.is_riichi && !pl.is_daburii;
    yf.dabururiichi = pl.is_daburii;
    yf.ippatsu = pl.is_ippatsu;
    yf.haiteiraoyue = stg.left_tile_count == 0;
    yf.rinshankaihou = pl.is_rinshan;
    yf.tenhou = false;
    yf.tiihou = false;

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
        stg.is_leader(pl.seat),
        stg.get_prevalent_wind(),
        stg.get_seat_wind(pl.seat),
        yf,
    ) {
        if !res.yaku.is_empty() {
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
    let pl_turn = &stg.players[stg.turn];
    if !pl.is_shown || pl.is_furiten || pl.is_furiten_other {
        return None;
    }

    let mut yf = YakuFlags::default();
    yf.riichi = pl.is_riichi && !pl.is_daburii;
    yf.dabururiichi = pl.is_daburii;
    yf.ippatsu = pl.is_ippatsu;

    // 和了牌を取得
    let mut t = Z8;
    let mut m_type = None;
    // 捨て牌に対する和了判定の場合
    if let Some(d) = pl_turn.discards.last() {
        if d.step == stg.step {
            t = d.tile;
            yf.houteiraoyui = stg.left_tile_count == 0;
        }
    }
    // 加槓(槍槓), 暗槓(国士無双)に対する和了判定の場合
    if t == Z8 {
        while let Some(m) = pl_turn.melds.last() {
            if m.step == stg.step {
                t = *m.tiles.last().unwrap();
                m_type = Some(m.type_);
                match m.type_ {
                    MeldType::Kakan => yf.chankan = true,
                    MeldType::Ankan => {}
                    _ => return None, // Chii, Pon, Minkan
                }
            }
        }
    }
    assert!(t != Z8); // ロンの和了判定は必ず打牌,加槓,暗槓のいずれか

    let mut hand = pl.hand.clone();
    hand[t.0][t.n()] += 1;
    if t.1 == 0 {
        hand[t.0][0] += 1;
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
        stg.is_leader(pl.seat),
        stg.get_prevalent_wind(),
        stg.get_seat_wind(pl.seat),
        yf,
    ) {
        if m_type == Some(MeldType::Ankan) {
            for y in &res.yaku {
                if y.name == "国士無双" || y.name == "国士無双十三面待ち" {
                    return Some(res);
                }
            }
        } else if !res.yaku.is_empty() {
            return Some(res);
        }
    }

    None
}

// 和了形である場合、最も高得点となるような役の組み合わせのSome(Result)を返却
// 和了形でない場合、Noneを返却
// 和了形でも無役の場合はResultの中身がyaku: [], pay_scores(0, 0, 0)となる。
pub fn evaluate_hand(
    hand: &TileTable,      // 手牌(鳴き以外)
    melds: &Vec<Meld>,     // 鳴き
    doras: &Vec<Tile>,     // ドラ表示牌 (注:ドラそのものではない)
    ura_doras: &Vec<Tile>, // 裏ドラ表示牌 リーチしていない場合は空
    winning_tile: Tile,    // 上がり牌
    is_self_drawn: bool,   // ツモ和了
    is_leader: bool,       // 親番
    prevalent_wind: Tnum,  // 場風 (東: 1, 南: 2, 西: 3, 北: 4)
    seat_wind: Tnum,       // 自風 (同上)
    yaku_flags: YakuFlags, // 和了形だった場合に自動的に付与される役(特殊条件役)のフラグ
) -> Option<WinContext> {
    let mut wins = vec![];

    // 和了(通常)
    let pm = parse_melds(melds);
    for mut ph in split_into_normal_win(hand).into_iter() {
        ph.append(&mut pm.clone());
        let ctx = Context::new(
            hand.clone(),
            ph,
            winning_tile,
            prevalent_wind,
            seat_wind,
            is_self_drawn,
            yaku_flags.clone(),
        );
        wins.push((ctx.get_fu(), ctx.get_yaku()));
    }

    // 和了(七対子)
    for ph in split_into_chiitoitsu_win(hand).into_iter() {
        let ctx = Context::new(
            hand.clone(),
            ph,
            winning_tile,
            prevalent_wind,
            seat_wind,
            is_self_drawn,
            yaku_flags.clone(),
        );
        wins.push((ctx.get_fu(), ctx.get_yaku()));
    }

    // 和了(国士無双)
    for ph in split_into_kokusimusou_win(hand).into_iter() {
        let ctx = Context::new(
            hand.clone(),
            ph,
            winning_tile,
            prevalent_wind,
            seat_wind,
            is_self_drawn,
            yaku_flags.clone(),
        );
        wins.push((ctx.get_fu(), ctx.get_yaku()));
    }

    if wins.is_empty() {
        return None; // 和了形以外
    }

    // 赤5の勘定
    let mut n_red = hand[0][0] + hand[1][0] + hand[2][0];
    for m in melds {
        for t in &m.tiles {
            if t.1 == 0 {
                n_red += 1;
            }
        }
    }

    let n_dora = count_dora(hand, melds, doras) + n_red;
    let n_ura_dora = if yaku_flags.riichi || yaku_flags.dabururiichi {
        count_dora(hand, melds, ura_doras)
    } else {
        0
    };

    let mut results = vec![];
    for (fu, (yaku, is_yakuman, mut fan_mag)) in wins {
        if !is_yakuman {
            fan_mag += n_dora + n_ura_dora;
        }
        let pay_scores = if yaku.is_empty() {
            (0, 0, 0) // 役無し
        } else {
            get_pay_scores(is_leader, is_yakuman, fu, fan_mag)
        };

        results.push(WinContext {
            fu,
            yaku,
            n_dora,
            n_ura_dora,
            is_yakuman,
            fan_mag,
            pay_scores,
        });
    }

    results.sort_by_key(|r| r.pay_scores.0);
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
            n_dora += dt[t.0][t.n()];
        }
    }

    n_dora
}
