use crate::model::*;
use crate::operator::operator::*;
use crate::util::stage_listener::StageListener;

#[derive(Clone)]
pub struct Null {}

impl Null {
    pub fn new() -> Self {
        Null {}
    }
}

impl Operator for Null {
    fn handle_operation(
        &mut self,
        _stage: &Stage,
        _seat: Seat,
        _operatons: &Vec<PlayerOperation>,
    ) -> PlayerOperation {
        panic!();
    }

    fn name(&self) -> String {
        "Null".to_string()
    }
}

impl StageListener for Null {}
