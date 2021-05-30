use crate::model::*;

use super::instance::*;
use super::operator::*;

use TileStateType::*;

pub fn create_operator(name: &str, _opts: &Vec<String>) -> Box<dyn Operator> {
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

pub fn count_left_tile(stage: &Stage, seat: Seat, tile: Tile) -> usize {
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
