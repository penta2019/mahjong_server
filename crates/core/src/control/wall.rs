use rand::prelude::*;

use crate::model::*;

pub fn create_wall(seed: u64, n_red5: usize) -> Vec<Tile> {
    assert!(n_red5 <= 4);
    let mut wall = Vec::new();
    for ti in 0..TYPE {
        for ni in 1..TNUM {
            if ti == TZ && ni > DR {
                break;
            }
            for n in 0..TILE {
                let ni2 = if ti != TZ && ni == 5 && n < n_red5 {
                    0
                } else {
                    ni
                }; // 赤5
                wall.push(Tile(ti, ni2));
            }
        }
    }

    let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(seed);
    wall.shuffle(&mut rng);
    wall
}

// デバッグ用に作為的な牌山を生成 指定がない場所はシード値に従ってランダムに生成
#[allow(dead_code)]
pub fn create_wall_debug(seed: u64, n_red5: usize) -> Vec<Tile> {
    assert!(n_red5 <= 4);
    let dora = vec![]; // 最大5
    let ura_dora = vec![]; // 最大5
    let replacement = vec![]; // 最大4
    let hands = [
        // 最大13x4
        vec![
            "m1", "m1", "m1", "m2", "m3", "m4", "m5", "m6", "m7", "m8", "m9", "m9", "m9",
        ], // seat0
        vec![], // seat1
        vec![], // seat2
        vec![], // seat3
    ];
    let deal = vec!["m1", "m0"]; // ツモ山 最初の牌は親番の14枚目

    let mut tt: TileTable = [[4; 10]; 4];
    tt[TM][5] = 4 - n_red5;
    tt[TM][0] = n_red5;
    tt[TP][5] = 4 - n_red5;
    tt[TP][0] = n_red5;
    tt[TS][5] = 4 - n_red5;
    tt[TS][0] = n_red5;
    tt[TZ][0] = 0;
    tt[TZ][8] = 0;
    tt[TZ][9] = 0;

    subtract_tile_table(&mut tt, &dora);
    subtract_tile_table(&mut tt, &ura_dora);
    subtract_tile_table(&mut tt, &replacement);
    for h in &hands {
        subtract_tile_table(&mut tt, h);
        assert!(h.len() < 14);
    }
    subtract_tile_table(&mut tt, &deal);

    // 余った牌をランダムにシャッフル
    let mut remain = Vec::new();
    for ti in 0..TYPE {
        for ni in 0..TNUM {
            while tt[ti][ni] != 0 {
                tt[ti][ni] -= 1;
                remain.push(Tile(ti, ni))
            }
        }
    }
    let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(seed);
    remain.shuffle(&mut rng);

    let mut wall = Vec::new();
    append_tiles_from_symbol(&mut wall, &dora);
    move_tiles(&mut remain, &mut wall, 5 - dora.len());
    append_tiles_from_symbol(&mut wall, &ura_dora);
    move_tiles(&mut remain, &mut wall, 5 - ura_dora.len());
    append_tiles_from_symbol(&mut wall, &replacement);
    move_tiles(&mut remain, &mut wall, 4 - replacement.len());
    for h in &hands {
        append_tiles_from_symbol(&mut wall, h);
        move_tiles(&mut remain, &mut wall, 13 - h.len());
    }
    append_tiles_from_symbol(&mut wall, &deal);
    move_tiles(&mut remain, &mut wall, 136 - 14 - 13 * 4 - deal.len());

    assert!(remain.is_empty());
    wall
}

fn subtract_tile_table(tt: &mut TileTable, tiles: &[&str]) {
    for tn in tiles {
        let t = Tile::from_symbol(tn);
        tt[t.0][t.1] -= 1;
    }
}

fn append_tiles_from_symbol(v: &mut Vec<Tile>, tiles: &[&str]) {
    for tn in tiles {
        let t = Tile::from_symbol(tn);
        v.push(t);
    }
}

fn move_tiles(source: &mut Vec<Tile>, target: &mut Vec<Tile>, count: usize) {
    for _ in 0..count {
        target.push(source.pop().unwrap());
    }
}

#[test]
fn test_debug_wall() {
    let wall = create_wall_debug(0, 1);
    for (i, t) in wall.iter().enumerate() {
        println!("{:2}: {}", i, t);
    }
}
