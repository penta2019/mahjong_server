use std::io::{stdout, Write};

use crate::model::*;
use crate::util::player_operation::*;

use PlayerOperation::*;

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
    ) -> (usize, usize) {
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
            let (a0, a1) = match c {
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
                    (0, enc_discard(Tile(ti, ni), false))
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

                    let mut ok = true;
                    let mut arg = [0usize; 2];
                    for i in 0..2 {
                        match v[i].trim().parse() {
                            Ok(n) => arg[i] = n,
                            Err(_) => {
                                println!("input must be usize");
                                ok = false;
                            }
                        }
                    }

                    if !ok {
                        continue;
                    }

                    (arg[0], arg[1])
                }
            };

            if !(a0 < ops.len()) {
                println!("invalid op_idx");
                continue;
            }

            match &ops[a0] {
                Discard(_) => {
                    let h = &stage.players[seat].hand;
                    let (t, _) = dec_discard(a1);
                    if t.0 > TZ || t.1 > 9 {
                        println!("Invalid tile: {:?}", t);
                        continue;
                    } else if h[t.0][t.1] == 0 {
                        println!("Tile not found: {}", t);
                        continue;
                    }
                }
                Chii(v) => {
                    if !check_vec_idx(v, a1) {
                        continue;
                    }
                }
                Pon(v) => {
                    if !check_vec_idx(v, a1) {
                        continue;
                    }
                }
                Ankan(v) => {
                    if !check_vec_idx(v, a1) {
                        continue;
                    }
                }
                Minkan(v) => {
                    if !check_vec_idx(v, a1) {
                        continue;
                    }
                }
                Kakan(v) => {
                    if !check_vec_idx(v, a1) {
                        continue;
                    }
                }
                Riichi(v) => {
                    if !check_vec_idx(v, a1) {
                        continue;
                    }
                }
                _ => {}
            }

            return (a0, a1);
        }
    }

    fn debug_string(&self) -> String {
        String::from("AlgoEmpty")
    }
}

fn check_vec_idx<T>(vec: &Vec<T>, idx: usize) -> bool {
    if idx < vec.len() {
        return true;
    }
    println!("invalid index");
    return false;
}
