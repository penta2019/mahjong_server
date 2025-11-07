use crate::model::*;

#[inline]
pub fn calc_seat_offset(base_seat: Seat, target_seat: Seat) -> Seat {
    (target_seat + SEAT - base_seat) % SEAT
}

#[inline]
pub fn calc_prevalent_wind(round: usize) -> Tnum {
    round % SEAT + 1 // WE | WS | WW | WN
}

#[inline]
pub fn calc_seat_wind(dealer: Seat, seat: Seat) -> Tnum {
    calc_seat_offset(dealer, seat) + 1 // WE | WS | WW | WN
}

// Stage用関数
#[inline]
pub fn is_dealer(stg: &Stage, seat: Seat) -> bool {
    seat == stg.dealer
}

#[inline]
pub fn get_prevalent_wind(stg: &Stage) -> Tnum {
    calc_prevalent_wind(stg.round)
}

#[inline]
pub fn get_seat_wind(stg: &Stage, seat: Seat) -> Tnum {
    calc_seat_wind(stg.dealer, seat)
}

pub fn get_scores(stg: &Stage) -> [Score; SEAT] {
    let mut scores = [0; SEAT];
    for s in 0..SEAT {
        scores[s] = stg.players[s].score;
    }
    scores
}

pub fn get_names(stg: &Stage) -> [String; SEAT] {
    let mut names = [""; SEAT];
    for s in 0..SEAT {
        names[s] = &stg.players[s].name;
    }
    names.map(|name| name.into())
}

// ダブル立直, 天和, 地和の判定用
pub fn is_no_meld_turn1(stg: &Stage, seat: Seat) -> bool {
    if !stg.players[seat].discards.is_empty() {
        false
    } else {
        stg.players
            .iter()
            .all(|pl| pl.melds.is_empty() && pl.nukidoras.is_empty())
    }
}

// TileTable
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

pub fn tiles_to_tile_table(tiles: &[Tile]) -> TileTable {
    let mut tt = TileTable::default();
    for &t in tiles {
        inc_tile(&mut tt, t);
    }
    tt
}

// ドラ表示牌のリストを受け取ってドラ評価値のテーブルを返却
pub fn create_dora_table(doras: &[Tile]) -> TileTable {
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
pub fn count_dora(hand: &TileTable, melds: &[Meld], doras: &[Tile]) -> usize {
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
