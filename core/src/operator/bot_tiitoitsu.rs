use crate::model::*;
use crate::util::operator::*;

use PlayerOperation::*;

#[derive(Clone)]
pub struct TiitoitsuBot {}

// 七対子Bot 試作
impl TiitoitsuBot {
    pub fn new() -> Self {
        Self {}
    }
}

impl Operator for TiitoitsuBot {
    fn handle_operation(
        &mut self,
        stage: &Stage,
        seat: Seat,
        ops: &Vec<PlayerOperation>,
    ) -> PlayerOperation {
        let h = &stage.players[seat].hand;

        if stage.turn == seat {
            // turn
            if ops.contains(&Tsumo) {
                return Tsumo;
            }

            let mut ones = vec![]; // 手牌に1枚のみある牌(left_count, Tile)
            for ti in 0..TYPE {
                for ni in 1..TNUM {
                    let t = Tile(ti, ni);
                    match h[ti][ni] {
                        0 | 2 => {}
                        3 | 4 => {
                            return Discard(vec![t]);
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
                return Discard(vec![ones[0].1]);
            }
        } else {
            // call
            if ops.contains(&Ron) {
                return Ron;
            }
        }
        Nop
    }

    fn debug_string(&self) -> String {
        "TiitoitsuBot".to_string()
    }
}
