use super::*;
use crate::util::parse_block::*;

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
        let h = &stage.players[seat].hand;

        if stage.turn == seat {
            // turn
            if acts.contains(&Action::tsumo()) {
                return Action::tsumo();
            }

            let mut ones = vec![]; // 手牌に1枚のみある牌(left_count, Tile)
            for ti in 0..TYPE {
                for ni in 1..TNUM {
                    let t = Tile(ti, ni);
                    match h[ti][ni] {
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

impl StageListener for TiitoitsuBot {}
