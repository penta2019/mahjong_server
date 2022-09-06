use super::parse::parse_into_chiitoitsu_win;
use crate::model::*;

// [完成形判定 (面子, 雀頭)]

// それぞれの牌種について"枚数を3で割った余り"と"余り数の集計"を返却
pub fn calc_mods_cnts(hand: &TileTable) -> ([usize; 4], [usize; 3]) {
    let mut mods = [0; TYPE];
    for ti in 0..TYPE {
        mods[ti] = hand[ti][1..10].iter().sum();
        mods[ti] %= 3;
    }

    let mut cnts = [0; 3];
    for ti in 0..TYPE {
        cnts[mods[ti]] += 1;
    }

    (mods, cnts)
}

// 雀頭+面子形で構成されているかの判定
pub fn is_sets_pair(tr: &TileRow, ti: Type) -> bool {
    !calc_pair_candidate(tr, ti).is_empty()
}

// 面子のみで構成されているかの判定
pub fn is_sets(tr: &TileRow, ti: Type) -> bool {
    let (mut n0, mut n1, mut n2);
    n0 = tr[1];
    n1 = tr[2];
    for i in 1..8 {
        n2 = tr[i + 2];
        let n = n0 % 3;
        if (ti == TZ && n != 0) || (n1 < n || n2 < n) {
            return false;
        }
        n0 = n1 - n;
        n1 = n2 - n;
    }
    n0 % 3 == 0 && n1 % 3 == 0
}

// 牌種が完成面子+雀頭の場合において雀頭候補となる牌を返す
// [1,4,7], [2,5,8], [3,6,9] のいずれか
pub fn calc_pair_candidate_index(tr: &TileRow) -> Vec<Tnum> {
    // 面子の和は3で割り切れるので余りの値によって雀頭候補を絞り込める
    let mut sum = 0;
    for i in 1..TNUM {
        sum += i * tr[i];
    }
    let mod3 = sum % 3;
    let mut pairs = vec![];
    for i in 1..4 {
        pairs.push(3 * i - mod3);
    }
    pairs
}

// 牌種が完成面子+雀頭のみで構成されている場合,雀頭のリストを返す.
// 基本的に1つだが,3113,3111113のような形の場合2つ
pub fn calc_pair_candidate(tr: &TileRow, ti: usize) -> Vec<Tile> {
    // 雀頭候補それぞれについて外してみた結果が完成面子になっているかをチェック
    let mut tr = *tr;
    let mut res = vec![];
    for ni in calc_pair_candidate_index(&tr) {
        if tr[ni] < 2 {
            continue;
        }
        tr[ni] -= 2;
        if is_sets(&tr, ti) {
            res.push(Tile(ti, ni));
        }
        tr[ni] += 2;
    }

    res
}

// 14 - (副露数) * 3 枚の手牌において和了形である場合,雀頭候補のリストを返却
pub fn calc_possibole_pairs(hand: &TileTable) -> Vec<Tile> {
    let (mods, cnts) = calc_mods_cnts(hand);
    let mut res = vec![];

    if cnts[1] != 0 || cnts[2] != 1 {
        return vec![];
    }

    for ti in 0..TYPE {
        if mods[ti] == 2 {
            let pairs = calc_pair_candidate(&hand[ti], ti);
            if pairs.is_empty() {
                return vec![];
            }
            res = pairs;
        } else {
            if !is_sets(&hand[ti], ti) {
                return vec![];
            }
        }
    }

    res
}

// [和了形判定]

// 通常形
pub fn is_normal_win(hand: &TileTable) -> bool {
    !calc_possibole_pairs(hand).is_empty()
}

// 七対子
pub fn is_chiitoitsu_win(hand: &TileTable) -> bool {
    !parse_into_chiitoitsu_win(hand).is_empty()
}

// 国士無双
pub fn is_kokushimusou_win(hand: &TileTable) -> bool {
    let mut count = 0;
    for ti in 0..TZ {
        if hand[ti][1] == 0 || hand[ti][9] == 0 {
            return false;
        }
        for ni in 2..9 {
            if hand[ti][ni] != 0 {
                return false;
            }
        }
        count += hand[ti][1] + hand[ti][9]
    }
    for ni in 1..8 {
        if hand[TZ][ni] == 0 {
            return false;
        }
        count += hand[TZ][ni]
    }

    count == 14
}

// [和了牌判定]
// 和了牌のリストを返却
// 聴牌していない場合は空のリストを返却

// 通常形
pub fn calc_tiles_to_normal_win(hand: &TileTable) -> Vec<Tile> {
    let (mods, cnts) = calc_mods_cnts(hand);
    let mut res = vec![];
    if cnts[1] == 0 && cnts[2] == 2 {
        // 雀頭候補が別種の牌(2つ)ある場合
        let mut ti_mod2 = vec![];
        for ti in 0..TYPE {
            if mods[ti] == 2 {
                ti_mod2.push(ti);
            } else {
                if !is_sets(&hand[ti], ti) {
                    return vec![];
                }
            }
        }
        for i in 0..2 {
            let (ti0, ti1) = (ti_mod2[i], ti_mod2[1 - i]);
            if is_sets_pair(&hand[ti0], ti0) {
                let mut tr = hand[ti1];
                for ni in 1..TNUM {
                    tr[ni] += 1;
                    if is_sets(&tr, ti1) {
                        res.push(Tile(ti1, ni));
                    }
                    tr[ni] -= 1;
                }
            }
        }
    }
    if cnts[1] == 1 && cnts[2] == 0 {
        // 雀頭候補が1つの牌種のみの場合
        for ti in 0..TYPE {
            if mods[ti] == 1 {
                let mut tr = hand[ti];
                for ni in 1..TNUM {
                    tr[ni] += 1;
                    if is_sets_pair(&tr, ti) {
                        res.push(Tile(ti, ni));
                    }
                    tr[ni] -= 1;
                }
            } else {
                if !is_sets(&hand[ti], ti) {
                    return vec![];
                }
            }
        }
    }

    res
}

// 七対子
pub fn calc_tiles_to_chiitoitsu_win(hand: &TileTable) -> Vec<Tile> {
    let mut res = vec![];
    let mut n_pair = 0;
    for ti in 0..TYPE {
        for ni in 1..TNUM {
            match hand[ti][ni] {
                1 => {
                    if res.is_empty() {
                        res.push(Tile(ti, ni));
                    }
                }
                2 => n_pair += 1,
                3 => return vec![],
                _ => {}
            }
        }
    }

    if n_pair == 6 {
        res
    } else {
        vec![]
    }
}

// 国士無双
pub fn calc_tiles_to_kokushimusou_win(hand: &TileTable) -> Vec<Tile> {
    let mut wt = None; // 所有していない么九牌
    let mut n_end = 0; // 么九牌の数
    let mut check = |ti: Type, ni: Tnum| {
        let n: usize = hand[ti][ni];
        n_end += n;
        match n {
            0 => {
                if wt != None {
                    // 二枚目が見つかった時点で聴牌していない
                    return true;
                }
                wt = Some(Tile(ti, ni));
                false
            }
            1 | 2 => false,
            _ => true,
        }
    };

    for ti in 0..TZ {
        if check(ti, 1) {
            return vec![];
        }
        if check(ti, 9) {
            return vec![];
        }
    }
    for ni in 1..8 {
        if check(TZ, ni) {
            return vec![];
        }
    }

    if n_end != 13 {
        return vec![];
    }

    if let Some(t) = wt {
        // 所有していない有効牌(么九牌)が一枚のみ存在する = 国士無双の和了牌
        vec![t]
    } else {
        // すべての么九牌を所持している = 国士無双十三面待ち
        let mut res = vec![];
        for ti in 0..TZ {
            res.push(Tile(ti, 1));
            res.push(Tile(ti, 9));
        }
        for ni in 1..8 {
            res.push(Tile(TZ, ni));
        }
        res
    }
}

// [聴牌捨て牌判定]
// ツモ番において聴牌となる打牌と待ちの組み合わせの一覧を返却
// 主にリーチ宣言が可能かどうかを確認する用途
// この関数群は手牌の赤5を考慮する

// 通常形
pub fn calc_discards_to_normal_tenpai(hand: &TileTable) -> Vec<(Tile, Vec<Tile>)> {
    let mut res = vec![];
    let mut hand = *hand;
    for ti in 0..TYPE {
        for ni in 1..TNUM {
            if hand[ti][ni] > 0 {
                hand[ti][ni] -= 1;
                let v = calc_tiles_to_normal_win(&hand);
                if !v.is_empty() {
                    res.push((Tile(ti, ni), v));
                }
                hand[ti][ni] += 1;
            }
        }
    }

    discards_with_red5(&hand, res)
}

// 七対子
pub fn calc_discards_to_chiitoitsu_tenpai(hand: &TileTable) -> Vec<(Tile, Vec<Tile>)> {
    let mut v1 = vec![]; // 一枚の牌
    let mut v2 = vec![]; // 二枚の牌 (完成対子)
    let mut v3 = vec![]; // 三枚の牌

    for ti in 0..TYPE {
        for ni in 1..TNUM {
            match hand[ti][ni] {
                0 => {}
                1 => {
                    if v1.len() > 1 {
                        return vec![];
                    }
                    v1.push(Tile(ti, ni));
                }
                2 => {
                    v2.push(Tile(ti, ni));
                }
                3 => {
                    if !v3.is_empty() {
                        return vec![];
                    }
                    v3.push(Tile(ti, ni));
                }
                _ => {
                    return vec![];
                }
            }
        }
    }

    let mut res = vec![];
    match v2.len() {
        0..=4 => {} // 聴牌未満
        5 => {
            if v1.len() == 1 && v3.len() == 1 {
                res = vec![(v3[0], vec![v1[0]])]
            }
        }
        6 => {
            assert!(v1.len() == 2);
            let (t0, t1) = (v1[0], v1[1]);
            res = vec![(t0, vec![t1]), (t1, vec![t0])]
        }
        7 => {
            // 和了形からあえて和了しない場合
            for t in v2 {
                res.push((t, vec![t]));
            }
        }
        _ => panic!(),
    }

    discards_with_red5(hand, res)
}

// 国士無双
pub fn calc_discards_to_kokushimusou_tenpai(hand: &TileTable) -> Vec<(Tile, Vec<Tile>)> {
    let mut n_end = 0;
    for ti in 0..TZ {
        if hand[ti][1] > 0 {
            n_end += 1;
        }
        if hand[ti][9] > 0 {
            n_end += 1;
        }
    }
    for ni in 1..8 {
        if hand[TZ][ni] > 0 {
            n_end += 1;
        }
    }

    if n_end < 12 {
        return vec![];
    }

    let mut res = vec![];
    let mut hand = *hand;
    for ti in 0..TYPE {
        for ni in 1..TNUM {
            if hand[ti][ni] > 0 {
                hand[ti][ni] -= 1;
                let v = calc_tiles_to_kokushimusou_win(&hand);
                if !v.is_empty() {
                    res.push((Tile(ti, ni), v));
                }
                hand[ti][ni] += 1;
            }
        }
    }

    discards_with_red5(&hand, res)
}

fn discards_with_red5(
    hand: &TileTable,
    discards: Vec<(Tile, Vec<Tile>)>,
) -> Vec<(Tile, Vec<Tile>)> {
    let mut res = vec![];
    for (t, wins) in discards {
        for t2 in tiles_with_red5(hand, t) {
            res.push((t2, wins.clone()))
        }
    }
    res
}
