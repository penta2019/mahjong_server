use mahjong_core::control::common::count_tile;

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
    stage: StageRef,
    seat: Seat,
}

// 七対子Bot 試作
impl TiitoitsuBot {
    pub fn from_config(config: Config) -> Self {
        Self {
            config,
            stage: StageRef::default(),
            seat: NO_SEAT,
        }
    }
}

impl Actor for TiitoitsuBot {
    fn init(&mut self, stage: StageRef, seat: Seat) {
        self.stage = stage;
        self.seat = seat;
    }

    fn select(&mut self, acts: &[Action], _tenpais: &[Tenpai]) -> ActionFuture {
        let stg = self.stage.lock().unwrap();

        let pl = &stg.players[self.seat];

        if stg.turn == self.seat {
            // turn
            if acts.contains(&Action::tsumo()) {
                return ready(Action::tsumo());
            }

            let mut ones = vec![]; // 手牌に1枚のみある牌(left_count, Tile)
            for ti in 0..TYPE {
                for ni in 0..TNUM {
                    let t = Tile(ti, ni);
                    match count_tile(&pl.hand, t) {
                        0 | 2 => {}
                        3 | 4 => {
                            return ready(Action::discard(t));
                        }
                        1 => {
                            ones.push((count_left_tile(&stg, self.seat, t), t));
                        }
                        _ => panic!(),
                    }
                }
            }

            // 1枚の牌で最も残り枚数が少ない牌から切る
            ones.sort();
            if !ones.is_empty() {
                return ready(Action::discard(ones[0].1));
            }
        } else {
            // call
            if acts.contains(&Action::ron()) {
                return ready(Action::ron());
            }
        }

        ready(Action::nop())
    }

    fn get_config(&self) -> &Config {
        &self.config
    }
}

impl Listener for TiitoitsuBot {}

pub fn count_left_tile(stg: &Stage, seat: Seat, tile: Tile) -> usize {
    use TileState::*;
    let mut n = 0;
    for &st in &stg.tile_states[tile.0][tile.1] {
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
