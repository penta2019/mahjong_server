use std::process::exit;

use crate::hand::{evaluate_hand, YakuFlags};
use crate::model::*;
use crate::util::common::*;

use crate::error;

#[derive(Debug)]
pub struct CalculatorApp {
    file_path: String,
    exp: String,
}

impl CalculatorApp {
    pub fn new(args: Vec<String>) -> Self {
        let mut app = Self {
            file_path: "".to_string(),
            exp: "".to_string(),
        };

        let mut it = args.iter();
        while let Some(s) = it.next() {
            match s.as_str() {
                "-f" => {
                    app.file_path = next_value(&mut it, "-f");
                }
                exp => {
                    if exp.starts_with("-") {
                        error!("unknown option: {}", exp);
                        exit(1);
                    }
                    app.exp = s.clone();
                }
            }
        }

        app
    }

    pub fn run(&self) {
        let calculator = Calculator::new(&self.exp);
        calculator.run();
    }
}

#[derive(Debug)]
struct Calculator {
    exp: String,
}

impl Calculator {
    fn new(exp: &str) -> Self {
        Self {
            exp: exp.to_string(),
        }
    }

    fn run(&self) {
        let mut hand = TileTable::default();
        let mut melds = vec![];
        let mut doras = vec![];
        let mut ura_doras = vec![];
        let mut win_tile = Z8;
        let mut is_drawn = true;
        let mut is_dealer = false;
        let mut prevalent_wind = 1;
        let mut seat_wind = 1;
        let mut yaku_flags = YakuFlags::default();

        let mut hand_raw = "".to_string();
        let mut melds_raw = vec![];
        for exp in self.exp.split(',') {
            if hand_raw == "" {
                hand_raw = exp.to_string();
            } else {
                melds_raw.push(exp.to_string());
            }
        }

        // println!("{:?}", hand_raw);
        // println!("{:?}", melds_raw);

        let undef: usize = 255;
        let mut ti = undef;
        for c in hand_raw.chars() {
            match c {
                'm' => ti = 0,
                'p' => ti = 1,
                's' => ti = 2,
                'z' => ti = 3,
                '+' => is_drawn = false,
                '0'..='9' => {
                    if ti == undef {
                        error!("tile number befor tile type");
                        exit(1);
                    }
                    let ni = c.to_digit(10).unwrap() as usize;
                    hand[ti][ni] += 1;
                    if ni == 0 {
                        hand[ti][5] += 1;
                    }
                    win_tile = Tile(ti, ni);
                }
                _ => {
                    error!("invalid char: '{}'", c);
                    exit(1);
                }
            }
        }

        println!("hand: {:?}", hand);
        println!("wintile: {:?}", win_tile);

        let res = evaluate_hand(
            &hand,
            &melds,
            &doras,
            &ura_doras,
            win_tile,
            is_drawn,
            is_dealer,
            prevalent_wind,
            seat_wind,
            yaku_flags,
        );

        println!("res: {:?}", res);
    }
}
