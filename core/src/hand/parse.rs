use crate::model::*;
use crate::util::common::cartesian_product;

use super::win::{calc_possibole_pairs, is_kokushimusou_win};

use SetPairType::*;

#[derive(Debug, Clone, Copy)]
pub enum SetPairType {
    Pair,    // 雀頭
    Shuntsu, // 順子
    Koutsu,  // 刻子
    Chii,    // チー
    Pon,     // ポン
    Minkan,  // 明槓 (大明槓 + 加槓)
    Ankan,   // 暗槓
}

// Tileは順子、チーの場合は先頭の牌
#[derive(Debug, Clone, Copy)]
pub struct SetPair(pub SetPairType, pub Tile);

pub type ParsedHand = Vec<SetPair>;

// 鳴きをSetPairに変換したリストを返却
pub fn parse_melds(melds: &Vec<Meld>) -> ParsedHand {
    let mut res = vec![];

    for m in melds {
        let mut t = m.tiles[0];
        if t.1 == 0 {
            t.1 = 5;
        }
        res.push(match m.type_ {
            MeldType::Chii => SetPair(Chii, t),
            MeldType::Pon => SetPair(Pon, t),
            MeldType::Minkan | MeldType::Kakan => SetPair(Minkan, t),
            MeldType::Ankan => SetPair(Ankan, t),
        });
    }

    res
}

// 牌種を順子と刻子に分解
// 三連刻の場合2通り(刻子3つ, 順子3つ)の分割が存在する　四連刻は役満(四暗刻)なので無視
// 予め分解可能であることを確認しておくこと(分解できない場合assertに失敗)
// TileRowが空(すべて0)の場合は分解可能とみなし[[]]を返却
fn parse_row_into_sets(tr: &TileRow, ti: usize) -> Vec<ParsedHand> {
    let mut ph = vec![];
    let (mut n0, mut n1, mut n2);

    n0 = tr[1];
    n1 = tr[2];
    for i in 1..8 {
        n2 = tr[i + 2];

        // 刻子
        if n0 >= 3 {
            ph.push(SetPair(Koutsu, Tile(ti, i)));
        }

        // 順子 (字牌はn=0となる)
        let n = n0 % 3;
        for _ in 0..n {
            ph.push(SetPair(Shuntsu, Tile(ti, i)))
        }
        n0 = n1 - n;
        n1 = n2 - n;
    }
    if n0 == 3 {
        ph.push(SetPair(Koutsu, Tile(ti, 8)));
    }
    if n1 == 3 {
        ph.push(SetPair(Koutsu, Tile(ti, 9)));
    }
    assert!(n0 % 3 == 0 && n1 % 3 == 0);

    if ti == TZ || ph.len() < 3 {
        return vec![ph];
    }

    // 三連刻チェック
    let (mut i, mut n) = (0, 0);
    for SetPair(tp, t) in &ph {
        if let Koutsu = tp {
            if i + n == t.1 {
                n += 1;
                if n == 3 {
                    break;
                }
            } else {
                i = t.1;
                n = 1;
            }
        }
    }

    // 三連刻なし
    if n != 3 {
        return vec![ph];
    }

    let mut ph2 = vec![];
    for &SetPair(tp, t) in &ph {
        if let Koutsu = tp {
            if i <= t.1 && t.1 < i + 3 {
                continue;
            }
        }
        ph2.push(SetPair(tp, t));
    }
    let sp = SetPair(Shuntsu, Tile(ti, i));
    ph2.push(sp);
    ph2.push(sp);
    ph2.push(sp);

    vec![ph, ph2]
}

// 手牌が完成形(七対子・国士無双は除く)なら面子+雀頭に分解して返却
pub fn parse_into_normal_win(hand: &TileTable) -> Vec<ParsedHand> {
    let pairs = calc_possibole_pairs(&hand);
    if pairs.is_empty() {
        return vec![];
    }

    let mut phs_list = vec![];

    // 雀頭を含む列
    let pair_ti = pairs[0].0;
    let mut tr = hand[pair_ti].clone();
    let mut phs = vec![];
    for pair in pairs {
        tr[pair.1] -= 2;
        let mut phs2 = parse_row_into_sets(&tr, pair_ti);
        tr[pair.1] += 2;
        for ph in &mut phs2 {
            ph.push(SetPair(Pair, pair));
        }
        phs.append(&mut phs2);
    }
    phs_list.push(phs);

    // 雀頭を含まない列
    for ti in 0..TYPE {
        if ti != pair_ti {
            phs_list.push(parse_row_into_sets(&hand[ti], ti));
        }
    }

    // それぞれの列の分割のすべての組み合わせ(直積)を求める
    let mut res = vec![];
    for v in cartesian_product(&phs_list) {
        let mut ph = vec![];
        for v2 in v {
            ph.extend(v2);
        }
        res.push(ph);
    }

    res
}

// 手牌が完成形(七対子)ならすべて対子に分解して返却
pub fn parse_into_chiitoitsu_win(hand: &TileTable) -> Vec<ParsedHand> {
    let mut res = vec![];
    for ti in 0..TYPE {
        for ni in 1..TNUM {
            let t = hand[ti][ni];
            if t == 0 {
                continue;
            } else if t == 2 {
                res.push(SetPair(Pair, Tile(ti, ni)));
            } else {
                return vec![];
            }
        }
    }

    if res.len() == 7 {
        vec![res]
    } else {
        vec![] // 鳴き有り
    }
}

// 手牌が完成形(国士無双)なら空のParsedHandが入ったリストを返却
pub fn parse_into_kokusimusou_win(hand: &TileTable) -> Vec<ParsedHand> {
    if is_kokushimusou_win(hand) {
        vec![vec![]]
    } else {
        vec![]
    }
}
