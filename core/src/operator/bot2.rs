use crate::model::*;
use crate::util::operator::*;
use crate::util::parse_block::*;

use PlayerOperation::*;

#[derive(Clone)]
pub struct Bot2 {}

impl Bot2 {
    pub fn new() -> Self {
        Bot2 {}
    }
}

impl Operator for Bot2 {
    fn handle_operation(
        &mut self,
        stage: &Stage,
        seat: Seat,
        ops: &Vec<PlayerOperation>,
    ) -> PlayerOperation {
        let h = &stage.players[seat].hand;

        if stage.turn == seat {
            if ops.contains(&Tsumo) {
                return Tsumo;
            }

            for ni in 1..=DR {
                if h[TZ][ni] != 0 && h[TZ][ni] != 3 {
                    return Discard(vec![Tile(TZ, ni)]);
                }
            }

            let bis = calc_block_info(h);
            let mut n_eff = 0;
            let mut t = Z8;
            for bi in bis {
                let ti = bi.tile.0;
                if ti == TZ {
                    continue;
                }
                let tr = calc_unnesesary_tiles(&h[ti], &bi, &stage.tile_remains[ti]);
                for ni in 1..TNUM {
                    if tr[ni] > n_eff {
                        n_eff = tr[ni];
                        t = Tile(ti, ni);
                    }
                }
            }
            if n_eff != 0 {
                return Discard(vec![t]);
            }
        } else {
            if ops.contains(&Ron) {
                return Ron;
            }
        }
        Nop
    }

    fn debug_string(&self) -> String {
        "Bot2".to_string()
    }
}
