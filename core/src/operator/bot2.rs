// use std::io::{stdout, Write};

use crate::model::*;
use crate::util::player_operation::*;

use PlayerOperation::*;
use TileStateType::*;

pub struct Bot2 {}

// 七対子Bot 試作
impl Bot2 {
    pub fn new() -> Self {
        Bot2 {}
    }
}

impl Operator for Bot2 {
    fn handle_operation(
        &mut self,
        stage: &Stage,
        seat: Seat,
        ops: &Vec<PlayerOperation>,
    ) -> (usize, usize) {
        let h = &stage.players[seat].hand;

        if stage.turn == seat {
        } else {
        }
        (0, 0) // Nop
    }

    fn debug_string(&self) -> String {
        "Bot2".to_string()
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
