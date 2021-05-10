use crate::model::*;
use crate::util::player_operation::*;

use PlayerOperation::*;
use TileStateType::*;

#[derive(Clone)]
pub struct Bot1 {}

// 七対子Bot 試作
impl Bot1 {
    pub fn new() -> Self {
        Bot1 {}
    }
}

impl Operator for Bot1 {
    fn handle_operation(
        &mut self,
        stage: &Stage,
        seat: Seat,
        ops: &Vec<PlayerOperation>,
    ) -> (usize, usize) {
        let h = &stage.players[seat].hand;

        if stage.turn == seat {
            // ツモ番
            // 七対子完成形の場合は和了る
            for (op_idx, op) in ops.iter().enumerate() {
                match op {
                    Tsumo => return (op_idx, 0),
                    _ => {}
                }
            }

            let mut ones = vec![]; // 手牌に1枚のみある牌(left_count, Tile)
            for ti in 0..TYPE {
                for ni in 1..TNUM {
                    let t = Tile(ti, ni);
                    match h[ti][ni] {
                        0 | 2 => {} // 何もしない
                        3 | 4 => {
                            return (0, enc_discard(t, false));
                        }
                        1 => {
                            ones.push((count_left_tile(stage, seat, t), t));
                        }
                        _ => panic!(),
                    }
                }
            }

            // 1枚の牌で最も残り枚数が少ない牌から切る
            ones.sort();
            if !ones.is_empty() {
                return (0, enc_discard(ones[0].1, false));
            }
        } else {
            // 他人のツモ番
            for (op_idx, op) in ops.iter().enumerate() {
                match op {
                    Ron => return (op_idx, 0),
                    _ => {}
                }
            }
        }
        (0, 0) // Nop
    }

    fn debug_string(&self) -> String {
        "Bot1".to_string()
    }
}

fn count_left_tile(stage: &Stage, seat: Seat, tile: Tile) -> usize {
    let mut n = 0;
    for &st in &stage.tile_states[tile.0][tile.1] {
        match st {
            U => {
                n += 1;
            }
            H(s) => {
                if s != seat {
                    n += 1;
                }
            }
            _ => {}
        }
    }
    n
}
