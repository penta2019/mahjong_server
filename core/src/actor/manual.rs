use super::*;
use crate::util::common::prompt;

pub struct ManualBuilder;

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
}

impl Manual {
    pub fn from_config(config: Config) -> Self {
        Self { config: config }
    }
}

impl Actor for Manual {
    fn select_action(&mut self, stage: &Stage, seat: Seat, acts: &Vec<Action>) -> Action {
        println!("{}", &stage.players[seat]);
        println!();
        if stage.turn == seat {
            println!("[Turn Action] select tile or action");
        } else {
            println!("[Call Action] select action");
        }
        for (idx, act) in acts.iter().enumerate() {
            println!("{} => {:?}", idx, act);
        }

        loop {
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
                    if stage.turn != seat {
                        println!("[Error] discard not allowed");
                        continue;
                    }

                    let ti = match c {
                        'm' => TM,
                        'p' => TP,
                        's' => TS,
                        'z' => TZ,
                        _ => panic!(),
                    };
                    let ni: Tnum = match chars.next().unwrap().to_digit(10) {
                        Some(n) => n as usize,
                        _ => {
                            println!("[Error] invalid tile symbol");
                            continue;
                        }
                    };

                    let h = &stage.players[seat].hand;
                    let t = Tile(ti, ni);
                    if t.0 > TZ || t.1 > 9 {
                        println!("[Error] invalid tile: {:?}", t);
                        continue;
                    } else if h[t.0][t.1] == 0 {
                        println!("[Error] tile not found: {}", t);
                        continue;
                    }

                    println!();
                    return Action::discard(Tile(ti, ni));
                }
                '!' => {
                    match &buf[1..] {
                        "print\n" => {
                            println!("{}", stage);
                        }
                        _ => {
                            println!("[Error] unknown command: {}", &buf[1..]);
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
                            println!("[Error] input must be number");
                            continue;
                        }
                    };
                    if n >= acts.len() {
                        println!("[Error] invalid action index");
                        continue;
                    }

                    println!();
                    return acts[n].clone();
                }
            };
        }
    }

    fn get_config(&self) -> &Config {
        &self.config
    }
}

impl StageListener for Manual {}
