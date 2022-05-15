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
        let seat = 0;

        // evaluate_hand params
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

        // parse hands
        {
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
        }

        // parse melds
        for meld_raw in &melds_raw {
            let undef: usize = 255;
            let mut ti = undef;
            let mut nis = vec![];
            let mut from = 0;
            let mut tiles = vec![];
            let mut froms = vec![];
            for c in meld_raw.chars() {
                match c {
                    'm' => ti = 0,
                    'p' => ti = 1,
                    's' => ti = 2,
                    'z' => ti = 3,
                    '+' => {
                        if froms.is_empty() {
                            error!("invalid '+' suffix");
                            exit(1);
                        }
                        let last = froms.len() - 1;
                        froms[last] = from % SEAT;
                    }
                    '0'..='9' => {
                        if ti == undef {
                            error!("tile number befor tile type");
                            exit(1);
                        }

                        from += 1;
                        let ni = c.to_digit(10).unwrap() as usize;
                        nis.push(if ni == 0 { 5 } else { ni });
                        tiles.push(Tile(ti, ni));
                        froms.push(seat);
                    }
                    _ => {
                        error!("invalid char: '{}'", c);
                        exit(1);
                    }
                }
            }

            nis.sort();
            let mut diffs = vec![];
            let mut ni0 = nis[0];
            for ni in &nis[1..] {
                diffs.push(ni - ni0);
                ni0 = *ni;
            }
            println!("diffs: {:?}", diffs);
            let meld_type = if diffs.len() == 2 && vec_count(&diffs, &1) == 2 {
                MeldType::Chi
            } else if diffs.len() == 2 && vec_count(&diffs, &0) == 2 {
                MeldType::Pon
            } else if diffs.len() == 3 && vec_count(&diffs, &0) == 3 {
                if vec_count(&froms, &seat) == 4 {
                    MeldType::Ankan
                } else {
                    MeldType::Minkan
                }
            } else {
                error!("invalid meld: '{}'", meld_raw);
                exit(1);
            };

            melds.push(Meld {
                step: 0,
                seat: seat,
                type_: meld_type,
                tiles: tiles,
                froms: froms,
            });
        }

        println!("hand: {:?}", hand);
        println!("melds: {:?}", melds);
        println!("wintile: {:?}", win_tile);

        if is_drawn {
            yaku_flags.menzentsumo = true;
            for m in &melds {
                if m.type_ != MeldType::Ankan {
                    yaku_flags.menzentsumo = false;
                }
            }
        }
        seat_wind = seat + 1;

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
