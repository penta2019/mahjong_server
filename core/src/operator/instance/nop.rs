use crate::model::*;
use crate::operator::operator::*;
use crate::util::stage_listener::StageListener;

#[derive(Clone)]
pub struct Nop {}

impl Nop {
    pub fn new() -> Self {
        Nop {}
    }
}

impl Operator for Nop {
    fn handle_operation(
        &mut self,
        _stage: &Stage,
        _seat: Seat,
        _operatons: &Vec<PlayerOperation>,
    ) -> PlayerOperation {
        Op::nop()
    }

    fn name(&self) -> String {
        "Nop".to_string()
    }
}

impl StageListener for Nop {}
