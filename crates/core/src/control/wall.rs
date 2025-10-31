use rand::prelude::*;

use super::{common::dec_tile, string::tiles_from_string};
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
        vec![],                                        // seat0
        tiles_from_string("m112233445566z1").unwrap(), // seat1
        tiles_from_string("p112233445566z1").unwrap(), // seat2
        tiles_from_string("s112233445566z1").unwrap(), // seat3
    ];
    let deal = vec![Tile(TZ, 1)]; // ツモ山 最初の牌は親番の14枚目

    let mut tt: TileTable = [[4; 10]; 4];
    tt[TM][0] = n_red5;
    tt[TP][0] = n_red5;
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
    println!("{tt:?}");
    for ti in 0..TYPE {
        for ni in 0..TNUM {
            while tt[ti][ni] != 0 {
                dec_tile(&mut tt, Tile(ti, ni));
                remain.push(Tile(ti, ni))
            }
        }
    }
    let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(seed);
    remain.shuffle(&mut rng);

    let mut wall = Vec::new();
    append_tiles(&mut wall, &dora);
    move_tiles(&mut remain, &mut wall, 5 - dora.len());
    append_tiles(&mut wall, &ura_dora);
    move_tiles(&mut remain, &mut wall, 5 - ura_dora.len());
    append_tiles(&mut wall, &replacement);
    move_tiles(&mut remain, &mut wall, 4 - replacement.len());
    for h in &hands {
        append_tiles(&mut wall, h);
        move_tiles(&mut remain, &mut wall, 13 - h.len());
    }
    append_tiles(&mut wall, &deal);
    move_tiles(&mut remain, &mut wall, 136 - 14 - 13 * 4 - deal.len());

    println!("{wall:?}");
    println!("{remain:?}");
    assert!(remain.is_empty());
    wall
}

fn subtract_tile_table(tt: &mut TileTable, tiles: &[Tile]) {
    for t in tiles {
        dec_tile(tt, *t);
    }
}

fn append_tiles(v: &mut Vec<Tile>, tiles: &[Tile]) {
    for t in tiles {
        v.push(*t);
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
