use rand::Rng;

use super::*;
use crate::controller::count_tile;

pub struct RandomDiscardBuilder;

impl ActorBuilder for RandomDiscardBuilder {
    fn get_default_config(&self) -> Config {
        Config {
            name: "RandomDiscard".to_string(),
            args: vec![],
        }
    }

    fn create(&self, config: Config) -> Box<dyn Actor> {
        Box::new(RandomDiscard::from_config(config))
    }
}

#[derive(Clone)]
pub struct RandomDiscard {
    config: Config,
    rng: rand::rngs::StdRng,
    seat: Seat,
}

impl RandomDiscard {
    pub fn from_config(config: Config) -> Self {
        RandomDiscard {
            config,
            rng: rand::SeedableRng::seed_from_u64(0),
            seat: NO_SEAT,
        }
    }
}

impl Actor for RandomDiscard {
    fn init(&mut self, seat: Seat) {
        self.seat = seat;
    }

    fn select_action(&mut self, stg: &Stage, _acts: &Vec<Action>, retry: i32) -> Option<Action> {
        assert!(retry == 0);

        if stg.turn != self.seat {
            return Some(Action::nop());
        }

        let pl = &stg.players[self.seat];
        let mut n: usize = self.rng.gen_range(0..13);
        loop {
            for ti in 0..TYPE {
                for ni in 1..TNUM {
                    let t = Tile(ti, ni);
                    let c = count_tile(&pl.hand, t);
                    if c > n {
                        return Some(Action::discard(Tile(ti, ni)));
                    } else {
                        n -= c;
                    }
                }
            }
        }
    }

    fn get_config(&self) -> &Config {
        &self.config
    }
}

impl Listener for RandomDiscard {}
