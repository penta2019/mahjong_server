// use std::io::{stdout, Write};

use crate::model::*;
use crate::util::player_operation::*;

use PlayerOperation::*;
use TileStateType::*;

#[derive(Clone)]
pub struct Bot2 {}

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
