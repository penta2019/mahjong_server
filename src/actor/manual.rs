use super::*;
use crate::etc::misc::prompt;
use crate::util::common::*;

use crate::error;

pub struct ManualBuilder;
use ActionType::*;

impl ActorBuilder for ManualBuilder {
    fn get_default_config(&self) -> Config {
        Config {
            name: "Manual".to_string(),
            args: vec![],
        }
    }

    fn create(&self, config: Config) -> Box<dyn Actor> {
        Box::new(Manual::from_config(config))
    }
}

#[derive(Clone)]
pub struct Manual {
    config: Config,
    seat: Seat,
}

impl Manual {
    pub fn from_config(config: Config) -> Self {
        Self {
            config,
            seat: NO_SEAT,
        }
    }
}

impl Actor for Manual {
    fn init(&mut self, seat: Seat) {
        self.seat = seat;
    }

    fn select_action(
        &mut self,
        stg: &Stage,
        acts: &Vec<Action>,
        _tenpais: &Vec<Tenpai>,
        retry: i32,
    ) -> Option<Action> {
        assert!(retry == 0);

        println!("{}", &stg.players[self.seat]);
        println!();
        if stg.turn == self.seat {
            println!("[Turn Action] select action or discard tile");
        } else {
            println!("[Call Action] select action");
        }
        for (idx, act) in acts.iter().enumerate() {
            println!("{} => {}", idx, act);
        }

        let mut riichi = false;
        loop {
            if riichi {
                print!("riichi ");
            }
            let buf = prompt();
            let mut chars = buf.chars();
            let c = if let Some(c) = chars.next() {
                c
            } else {
                println!();
                continue;
            };
            match c {
                'm' | 'p' | 's' | 'z' => {
                    if stg.turn != self.seat {
                        error!("discard not allowed");
                        continue;
                    }

                    let ti = tile_type_from_char(c).unwrap();
                    let ni = match tile_number_from_char(chars.next().unwrap()) {
                        Ok(n) => n,
                        Err(_) => {
                            error!("invalid tile symbol");
                            continue;
                        }
                    };

                    let h = &stg.players[self.seat].hand;
                    let t = Tile(ti, ni);
                    if t.0 > TZ || t.1 > 9 {
                        error!("invalid tile: {}", t);
                        continue;
                    } else if h[t.0][t.1] == 0 {
                        error!("tile not found: {}", t);
                        continue;
                    }

                    if riichi {
                        if let Some(a) = acts.iter().find(|a| a.action_type == Riichi) {
                            if a.tiles.contains(&t) {
                                println!();
                                return Some(Action::riichi(t));
                            } else {
                                error!("invalid Riichi tile");
                            }
                        } else {
                            panic!();
                        }
                    } else {
                        if let Some(a) = acts.iter().find(|a| a.action_type == Discard) {
                            if !a.tiles.contains(&t) {
                                println!();
                                return Some(Action::discard(t));
                            } else {
                                error!("restricted tile after Chi or Pon");
                            }
                        } else {
                            error!("Discard is not allowed");
                        }
                    }
                }
                '!' => {
                    match &buf[1..] {
                        "print\n" => {
                            println!("{}", stg);
                        }
                        _ => {
                            error!("unknown command: {}", &buf[1..]);
                        }
                    }
                    continue;
                }
                '\n' => {
                    continue;
                }
                _ => {
                    let n: usize = match buf.trim().parse() {
                        Ok(n) => n,
                        Err(_) => {
                            error!("input must be a number or tile symbol");
                            continue;
                        }
                    };
                    if n >= acts.len() {
                        error!("invalid action index");
                        continue;
                    }

                    match acts[n].action_type {
                        Discard => {
                            println!("please select tile");
                        }
                        Riichi => {
                            riichi = true;
                        }
                        _ => {
                            println!();
                            return Some(acts[n].clone());
                        }
                    }
                }
            };
        }
    }

    fn get_config(&self) -> &Config {
        &self.config
    }
}

impl Listener for Manual {}
