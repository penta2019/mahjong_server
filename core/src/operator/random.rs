use rand::Rng;

use crate::model::*;
use crate::util::player_operation::*;

pub struct RandomDiscardOperator {
    rng: rand::rngs::StdRng,
}

impl RandomDiscardOperator {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: rand::SeedableRng::seed_from_u64(seed),
        }
    }
}

impl Operator for RandomDiscardOperator {
    fn handle_operation(
        &mut self,
        stage: &Stage,
        seat: Seat,
        ops: &Vec<PlayerOperation>,
    ) -> (usize, usize) {
        let h = &stage.players[seat].hand;
        let mut n: u32 = self.rng.gen_range(0, 13);
        loop {
            for ti in 0..TYPE {
                for ni in 1..TNUM {
                    if h[ti][ni] > 0 {
                        if n == 0 {
                            return (0, enc_discard(Tile(ti, ni), false));
                        }
                        n -= 1;
                    }
                }
            }
        }
    }

    fn debug_string(&self) -> String {
        String::from("RandomDiscardOperator")
    }
}
