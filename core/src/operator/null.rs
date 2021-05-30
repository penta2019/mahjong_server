use crate::model::*;
use crate::util::operator::*;
use crate::util::stage_listener::StageListener;

#[derive(Clone)]
pub struct NullOperator {}

impl NullOperator {
    pub fn new() -> Self {
        NullOperator {}
    }
}

impl Operator for NullOperator {
    fn handle_operation(
        &mut self,
        _stage: &Stage,
        _seat: Seat,
        _operatons: &Vec<PlayerOperation>,
    ) -> PlayerOperation {
        panic!();
    }

    fn name(&self) -> String {
        "NullOperator".to_string()
    }
}

impl StageListener for NullOperator {}
