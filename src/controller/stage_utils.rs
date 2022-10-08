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

pub fn tiles_from_tile_table(tt: &TileTable) -> Vec<Tile> {
    let mut hand = vec![];
    for ti in 0..TYPE {
        for ni in 1..TNUM {
            // 赤5
            if ni == 5 {
                for _ in 0..tt[ti][0] {
                    hand.push(Tile(ti, 0));
                }
            }

            for _ in 0..tt[ti][ni] {
                hand.push(Tile(ti, ni));
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
