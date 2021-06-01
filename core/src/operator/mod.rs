pub mod bot2;
pub mod manual;
pub mod mjai;
pub mod nop;
pub mod null;
pub mod random;
pub mod tiitoitsu;

use crate::controller::operator::Operator;
use crate::controller::stage_listener::StageListener;
use crate::model::*;

pub fn create_operator(
    name: &str,
    _opts: &Vec<String>,
) -> Box<dyn crate::controller::operator::Operator> {
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
