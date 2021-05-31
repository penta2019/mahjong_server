use std::fmt;

use crate::model::*;
use crate::util::stage_listener::StageListener;

// Operator trait
pub trait Operator: StageListener + OperatorClone + Send {
    fn set_seat(&mut self, _: Seat) {}
    fn handle_operation(
        &mut self,
        stage: &Stage,
        seat: Seat,
        operatons: &Vec<PlayerOperation>,
    ) -> PlayerOperation;
    fn name(&self) -> String;
}

impl fmt::Debug for dyn Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

// https://stackoverflow.com/questions/30353462/how-to-clone-a-struct-storing-a-boxed-trait-object
pub trait OperatorClone {
    fn clone_box(&self) -> Box<dyn Operator>;
}

impl<T> OperatorClone for T
where
    T: 'static + Operator + Clone,
{
    fn clone_box(&self) -> Box<dyn Operator> {
        Box::new(self.clone())
    }
}
