use crate::etc::misc::vec_count;
use crate::model::*;

#[inline]
pub fn is_dealer(stg: &Stage, seat: Seat) -> bool {
    seat == stg.dealer
}

#[inline]
pub fn get_prevalent_wind(stg: &Stage) -> Tnum {
    stg.round % SEAT + 1 // WE | WS | WW | WN
}

#[inline]
pub fn get_seat_wind(stg: &Stage, seat: Seat) -> Tnum {
    (seat + SEAT - stg.dealer) % SEAT + 1 // WE | WS | WW | WN
}

pub fn get_scores(stg: &Stage) -> [Score; SEAT] {
    let mut scores = [0; SEAT];
    for s in 0..SEAT {
        scores[s] = stg.players[s].score;
    }
    scores
}

pub fn count_tile(tt: &TileTable, t: Tile) -> usize {
    if t.1 == 5 {
        tt[t.0][t.1] - tt[t.0][0]
    } else {
        tt[t.0][t.1]
    }
}

pub fn inc_tile(tt: &mut TileTable, tile: Tile) {
    let t = tile;
    tt[t.0][t.1] += 1;
    if t.1 == 0 {
        // 0は赤5のフラグなので本来の5をたてる
        tt[t.0][5] += 1;
    }
}

pub fn dec_tile(tt: &mut TileTable, tile: Tile) {
    let t = tile;
    tt[t.0][t.1] -= 1;
    if t.1 == 0 {
        tt[t.0][5] -= 1;
    }
    assert!(tt[t.0][5] != 0 || tt[t.0][0] == 0);
}

pub fn tiles_from_tile_table(tt: &TileTable) -> Vec<Tile> {
    let mut hand = vec![];
    for ti in 0..TYPE {
        for ni in 1..TNUM {
            for c in 0..tt[ti][ni] {
                if ti != TZ && ni == 5 && c < tt[ti][0] {
                    hand.push(Tile(ti, 0)); // 赤5
                } else {
                    hand.push(Tile(ti, ni));
                }
            }
        }
    }
    hand
}

pub fn tiles_to_tile_table(tiles: &Vec<Tile>) -> TileTable {
    let mut tt = TileTable::default();
    for &t in tiles {
        inc_tile(&mut tt, t);
    }
    tt
}

// ドラ表示牌のリストを受け取ってドラ評価値のテーブルを返却
pub fn create_dora_table(doras: &Vec<Tile>) -> TileTable {
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

// ドラ表示牌によるのドラの数を勘定
pub fn count_dora(hand: &TileTable, melds: &Vec<Meld>, doras: &Vec<Tile>) -> usize {
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

pub fn tiles_with_red5(tt: &TileTable, t: Tile) -> Vec<Tile> {
    if tt[t.0][t.1] == 0 {
        return vec![];
    }

    let Tile(ti, ni) = t;
    let tr = tt[ti];
    if ni != 5 {
        return vec![t]; // 5ではない場合
    }
    if tr[0] == 0 {
        return vec![t]; // 通常5しかない場合
    }
    if tr[0] == tr[5] {
        return vec![Tile(ti, 0)]; // 赤5しかない場合
    }
    vec![t, Tile(ti, 0)] // 通常5と赤5の両方がある場合
}

pub fn tiles_from_string(exp: &str) -> Result<Vec<Tile>, String> {
    let mut tiles = vec![];
    let undef: usize = 255;
    let mut ti = undef;
    for c in exp.chars() {
        match c {
            'm' => ti = 0,
            'p' => ti = 1,
            's' => ti = 2,
            'z' => ti = 3,
            '0'..='9' => {
                if ti == undef {
                    return Err(format!("tile number befor tile type"));
                }
                let ni = c.to_digit(10).unwrap() as usize;
                tiles.push(Tile(ti, ni));
            }
            _ => {
                return Err(format!("invalid char: '{}'", c));
            }
        }
    }
    Ok(tiles)
}

pub fn meld_from_string(exp: &str) -> Result<Meld, String> {
    let undef: usize = 255;
    let seat = 0; // 点数計算する上で座席の番号は関係ないので0で固定
    let mut ti = undef;
    let mut nis = vec![];

    let mut from = 0;
    let mut tiles = vec![];
    let mut froms = vec![];
    for c in exp.chars() {
        match c {
            'm' => ti = 0,
            'p' => ti = 1,
            's' => ti = 2,
            'z' => ti = 3,
            '+' => {
                if froms.is_empty() {
                    return Err("invalid '+' suffix".to_string());
                }
                let last = froms.len() - 1;
                froms[last] = from % SEAT;
            }
            '0'..='9' => {
                if ti == undef {
                    return Err("tile number befor tile type".to_string());
                }

                from += 1;
                let ni = c.to_digit(10).unwrap() as usize;
                nis.push(if ni == 0 { 5 } else { ni });
                tiles.push(Tile(ti, ni));
                froms.push(seat);
            }
            _ => {
                return Err(format!("invalid char: '{}'", c));
            }
        }
    }

    nis.sort();
    let mut diffs = vec![];
    let mut ni0 = nis[0];
    for ni in &nis[1..] {
        diffs.push(ni - ni0);
        ni0 = *ni;
    }

    let meld_type = if diffs.len() == 2 && vec_count(&diffs, &1) == 2 {
        MeldType::Chi
    } else if diffs.len() == 2 && vec_count(&diffs, &0) == 2 {
        MeldType::Pon
    } else if diffs.len() == 3 && vec_count(&diffs, &0) == 3 {
        if vec_count(&froms, &seat) == 4 {
            MeldType::Ankan
        } else {
            MeldType::Minkan
        }
    } else {
        return Err(format!("invalid meld: '{}'", exp));
    };

    Ok(Meld {
        step: 0,
        seat,
        meld_type,
        tiles,
        froms,
    })
}

pub fn wind_from_char(c: char) -> Result<Index, String> {
    Ok(match c {
        'E' => 1,
        'S' => 2,
        'W' => 3,
        'N' => 4,
        _ => return Err(format!("invalid wind symbol: {}", c)),
    })
}

#[test]
fn test_tiletable() {
    let hand_str = "p34777s1230567z66";
    let hand = tiles_from_string(&hand_str).unwrap();
    let tt = tiles_to_tile_table(&hand);
    let hand2 = tiles_from_tile_table(&tt);
    assert_eq!(hand, hand2);
}
