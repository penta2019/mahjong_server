use super::*;
use crate::util::parse_block::*;

pub struct Bot2Builder;

impl OperatorBuilder for Bot2Builder {
    fn get_default_config(&self) -> Config {
        Config {
            name: "Bot2".to_string(),
            args: vec![],
        }
    }

    fn create(&self, config: Config) -> Box<dyn Operator> {
        Box::new(Bot2::from_config(config))
    }
}

#[derive(Clone)]
pub struct Bot2 {
    config: Config,
}

impl Bot2 {
    pub fn from_config(config: Config) -> Self {
        Self { config: config }
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
            if ops.contains(&Op::tsumo()) {
                return Op::tsumo();
            }

            for ni in 1..=DR {
                if h[TZ][ni] != 0 && h[TZ][ni] != 3 {
                    return Op::discard(Tile(TZ, ni));
                }
            }

            let bis = calc_block_info(h);

            // 孤立牌があれば一番端から切る
            let mut dist = 0;
            let mut t = Z8;
            for bi in &bis {
                if bi.num == 1 {
                    let d = std::cmp::min(10 - bi.tile.1, bi.tile.1);
                    if d > dist {
                        dist = d;
                        t = bi.tile;
                    }
                }
            }
            if t != Z8 {
                return Op::discard(t);
            }

            // 有効牌が一番多くなるような不要牌を探して切る
            let mut n_eff = 0;
            let mut t = Z8;
            for bi in &bis {
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
            if t != Z8 {
                return Op::discard(t);
            }
        } else {
            if ops.contains(&Op::ron()) {
                return Op::ron();
            }
        }

        Op::nop()
    }

    fn get_config(&self) -> &Config {
        &self.config
    }
}

impl StageListener for Bot2 {}
