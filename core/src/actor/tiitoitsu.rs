use super::*;

pub struct TiitoitsuBotBuilder;

impl ActorBuilder for TiitoitsuBotBuilder {
    fn get_default_config(&self) -> Config {
        Config {
            name: "TiitoitsuBot".to_string(),
            args: vec![],
        }
    }

    fn create(&self, config: Config) -> Box<dyn Actor> {
        Box::new(TiitoitsuBot::from_config(config))
    }
}

#[derive(Clone)]
pub struct TiitoitsuBot {
    config: Config,
}

// 七対子Bot 試作
impl TiitoitsuBot {
    pub fn from_config(config: Config) -> Self {
        Self { config: config }
    }
}

impl Actor for TiitoitsuBot {
    fn select_action(&mut self, stage: &Stage, seat: Seat, acts: &Vec<Action>) -> Action {
        let pl = &stage.players[seat];

        if stage.turn == seat {
            // turn
            if acts.contains(&Action::tsumo()) {
                return Action::tsumo();
            }

            let mut ones = vec![]; // 手牌に1枚のみある牌(left_count, Tile)
            for ti in 0..TYPE {
                for ni in 0..TNUM {
                    let t = Tile(ti, ni);
                    match pl.count_tile(t) {
                        0 | 2 => {}
                        3 | 4 => {
                            return Action::discard(t);
                        }
                        1 => {
                            ones.push((count_left_tile(stage, seat, t), t));
                        }
                        _ => panic!(),
                    }
                }
            }

            // 1枚の牌で最も残り枚数が少ない牌から切る
            ones.sort();
            if !ones.is_empty() {
                return Action::discard(ones[0].1);
            }
        } else {
            // call
            if acts.contains(&Action::ron()) {
                return Action::ron();
            }
        }

        Action::nop()
    }

    fn get_config(&self) -> &Config {
        &self.config
    }
}

impl EventListener for TiitoitsuBot {}

pub fn count_left_tile(stage: &Stage, seat: Seat, tile: Tile) -> usize {
    use TileStateType::*;
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
