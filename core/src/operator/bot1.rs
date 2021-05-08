use std::io::{stdout, Write};

use crate::model::*;
use crate::util::player_operation::*;

use PlayerOperation::*;

pub struct Bot1 {}

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
        let pl = &stage.players[seat];

        (0, 0)
    }

    fn debug_string(&self) -> String {
        "Bot1".to_string()
    }
}
