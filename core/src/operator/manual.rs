use std::io::{stdout, Write};

use crate::model::*;
use crate::util::player_operation::*;

use PlayerOperation::*;

#[derive(Clone)]
pub struct ManualOperator {}

impl ManualOperator {
    pub fn new() -> Self {
        Self {}
    }
}

impl Operator for ManualOperator {
    fn handle_operation(
        &mut self,
        stage: &Stage,
        seat: Seat,
        ops: &Vec<PlayerOperation>,
    ) -> PlayerOperation {
        println!("{}", &stage.players[seat]);
        if stage.turn == seat {
            println!("{:?}", ops);
        } else {
            println!(
                "seat{}: {} => {:?}",
                stage.turn,
                stage.last_tile.unwrap().1,
                ops
            );
        }

        loop {
            print!("> ");
            stdout().flush().unwrap();
            let mut buf = String::new();
            std::io::stdin().read_line(&mut buf).ok();

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
                    let ni: usize = match chars.next().unwrap().to_digit(10) {
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
                    return Discard(vec![Tile(ti, ni)]);
                }
                '!' => {
                    match &buf[1..] {
                        "print\n" => {
                            stage.print();
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
                    let v: Vec<&str> = buf.split(' ').collect();
                    if v.len() != 2 {
                        println!("input must be 2 numbers");
                        continue;
                    }

                    let mut arg = [0usize; 2];
                    for i in 0..2 {
                        match v[i].trim().parse() {
                            Ok(n) => arg[i] = n,
                            Err(_) => {
                                println!("input must be number");
                                continue;
                            }
                        }
                    }

                    let (a0, a1) = (arg[0], arg[1]);
                    if !(a0 < ops.len()) {
                        println!("invalid op_idx");
                        continue;
                    }

                    match &ops[a0] {
                        Nop => return Nop,
                        Discard(v) => {
                            let h = &stage.players[seat].hand;
                            let t = v[0];
                            if t.0 > TZ || t.1 > 9 {
                                println!("Invalid tile: {:?}", t);
                                continue;
                            } else if h[t.0][t.1] == 0 {
                                println!("Tile not found: {}", t);
                                continue;
                            }
                            return Discard(vec![t]);
                        }
                        Ankan(v) => {
                            if !check_vec_idx(v, a1) {
                                continue;
                            }
                            return Ankan(vec![v[a1]]);
                        }
                        Kakan(v) => {
                            if !check_vec_idx(v, a1) {
                                continue;
                            }
                            return Kakan(vec![v[a1]]);
                        }
                        Riichi(v) => {
                            if !check_vec_idx(v, a1) {
                                continue;
                            }
                            return Riichi(vec![v[a1]]);
                        }
                        Tsumo => return Tsumo,
                        Kyushukyuhai => return Kyushukyuhai,
                        Kita => return Kita,
                        Chii(v) => {
                            if !check_vec_idx(v, a1) {
                                continue;
                            }
                            return Chii(vec![v[a1]]);
                        }
                        Pon(v) => {
                            if !check_vec_idx(v, a1) {
                                continue;
                            }
                            return Pon(vec![v[a1]]);
                        }
                        Minkan(v) => {
                            if !check_vec_idx(v, a1) {
                                continue;
                            }
                            return Minkan(vec![v[a1]]);
                        }
                        Ron => return Ron,
                    }
                }
            };
        }
    }

    fn debug_string(&self) -> String {
        "ManualOperator".to_string()
    }
}

fn check_vec_idx<T>(vec: &Vec<T>, idx: usize) -> bool {
    if idx < vec.len() {
        return true;
    }
    println!("invalid index");
    return false;
}
