use rand::Rng;

use crate::model::*;
use crate::operator::operator::*;
use crate::util::stage_listener::StageListener;

#[derive(Clone)]
pub struct RandomDiscard {
    rng: rand::rngs::StdRng,
}

impl RandomDiscard {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: rand::SeedableRng::seed_from_u64(seed),
        }
    }
}

impl Operator for RandomDiscard {
    fn handle_operation(
        &mut self,
        stage: &Stage,
        seat: Seat,
        _ops: &Vec<PlayerOperation>,
    ) -> PlayerOperation {
        if stage.turn != seat {
            return Op::nop();
        }

        let h = &stage.players[seat].hand;
        let mut n: u32 = self.rng.gen_range(0, 13);
        loop {
            for ti in 0..TYPE {
                for ni in 1..TNUM {
                    if h[ti][ni] > 0 {
                        if n == 0 {
                            return Op::discard(Tile(ti, ni));
                        }
                        n -= 1;
                    }
                }
            }
        }
    }

    fn name(&self) -> String {
        "RandomDiscard".to_string()
    }
}

impl StageListener for RandomDiscard {}
