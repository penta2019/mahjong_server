pub mod bot2;
pub mod manual;
pub mod mjai;
pub mod nop;
pub mod null;
pub mod random;
pub mod tiitoitsu;

use std::fmt;

use crate::controller::stage_listener::StageListener;
use crate::model::*;

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

pub fn create_operator(name: &str) -> Box<dyn Operator> {
    match name {
        "" => {
            println!("Operator name is not specified. Uses 'Nop' Operator");
            Box::new(nop::Nop::new())
        }
        "Bot2" => Box::new(bot2::Bot2::new()),
        "Manual" => Box::new(manual::Manual::new()),
        "MjaiEndpoint" => Box::new(mjai::MjaiEndpoint::new("127.0.0.1:11601")),
        // "Null" => Box::new(null::Null::new()),
        "Nop" => Box::new(nop::Nop::new()),
        "RandomDiscard" => Box::new(random::RandomDiscard::new(0)),
        "TiitoitsuBot" => Box::new(tiitoitsu::TiitoitsuBot::new()),
        _ => {
            println!("Unknown operator name: {}", name);
            std::process::exit(0);
        }
    }
}
