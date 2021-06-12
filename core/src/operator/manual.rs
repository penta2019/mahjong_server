use std::io::{stdout, Write};

use super::*;

pub struct ManualBuilder;

impl OperatorBuilder for ManualBuilder {
    fn get_default_config(&self) -> Config {
        Config {
            name: "Manual".to_string(),
            args: vec![],
        }
    }

    fn create(&self, config: Config) -> Box<dyn Operator> {
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

impl Operator for Manual {
    fn handle_operation(
        &mut self,
        stage: &Stage,
        seat: Seat,
        ops: &Vec<PlayerOperation>,
    ) -> PlayerOperation {
        println!("{}", &stage.players[seat]);
        println!();
        if stage.turn == seat {
            println!("[Turn Operation] select tile or operation");
        } else {
            println!("[Call Operation] select operation");
        }
        for (idx, op) in ops.iter().enumerate() {
            println!("{} => {:?}", idx, op);
        }

        loop {
            print!("> ");
            stdout().flush().unwrap();
            let mut buf = String::new();
            std::io::stdin().read_line(&mut buf).ok();
            println!();

            let mut chars = buf.chars();
            let c = chars.next().unwrap();
            match c {
                'm' | 'p' | 's' | 'z' => {
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
                            println!("invalid tile symbol");
                            continue;
                        }
                    };

                    let h = &stage.players[seat].hand;
                    let t = Tile(ti, ni);
                    if t.0 > TZ || t.1 > 9 {
                        println!("Invalid tile: {:?}", t);
                        continue;
                    } else if h[t.0][t.1] == 0 {
                        println!("Tile not found: {}", t);
                        continue;
                    }
                    return Op::discard(Tile(ti, ni));
                }
                '!' => {
                    match &buf[1..] {
                        "print\n" => {
                            println!("{}", stage);
                        }
                        _ => {
                            println!("Unknown command: {}", &buf[1..]);
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
                            println!("input must be number");
                            continue;
                        }
                    };
                    if n >= ops.len() {
                        println!("invalid operation index");
                        continue;
                    }

                    return ops[n].clone();
                }
            };
        }
    }

    fn get_config(&self) -> &Config {
        &self.config
    }
}

impl StageListener for Manual {}
