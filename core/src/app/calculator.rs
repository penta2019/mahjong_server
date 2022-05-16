use crate::hand::{evaluate_hand, YakuFlags};
use crate::model::*;
use crate::util::common::*;

use crate::error_exit;

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
                        error_exit!("unknown option: {}", exp);
                    }
                    app.exp = s.clone();
                }
            }
        }

        app
    }

    pub fn run(&self) {
        let mut calculator = Calculator::new();
        calculator.parse(&self.exp);
        calculator.run();
    }
}

#[derive(Debug)]
struct Calculator {
    seat: Seat,
    // evaluate_hand params
    hand: TileTable,
    melds: Vec<Meld>,
    doras: Vec<Tile>,
    ura_doras: Vec<Tile>,
    win_tile: Tile,
    is_drawn: bool,
    is_dealer: bool,
    prevalent_wind: Index,
    seat_wind: Index,
    yaku_flags: YakuFlags,
}

impl Calculator {
    fn new() -> Self {
        Self {
            seat: 0,
            hand: TileTable::default(),
            melds: vec![],
            doras: vec![],
            ura_doras: vec![],
            win_tile: Z8,
            is_drawn: true,
            is_dealer: true,
            prevalent_wind: 1,
            seat_wind: 1,
            yaku_flags: YakuFlags::default(),
        }
    }

    fn parse(&mut self, input: &str) {
        let input = input.replace(" ", "");
        let exps: Vec<&str> = input.split("/").collect();

        if exps.len() == 0 {
            error_exit!("input is empty");
        }

        if let Some(exp) = exps.get(1) {
            self.parse_stage_info(exp); // 副露のパースに座席情報が必要なので最初に実行
        };
        if let Some(exp) = exps.get(0) {
            self.parse_hand_meld(exp);
        };
        if let Some(exp) = exps.get(2) {
            self.parse_yaku_flags(exp);
        };
        if let Some(exp) = exps.get(3) {
            self.parse_score_verify(exp);
        }

        println!("hand: {:?}", self.hand);
        println!("melds: {:?}", self.melds);
        println!("wintile: {:?}", self.win_tile);
    }

    fn run(&self) {
        let res = evaluate_hand(
            &self.hand,
            &self.melds,
            &self.doras,
            &self.ura_doras,
            self.win_tile,
            self.is_drawn,
            self.is_dealer,
            self.prevalent_wind,
            self.seat_wind,
            &self.yaku_flags,
        );

        println!("res: {:?}", res);
    }

    fn parse_stage_info(&mut self, exp: &str) {
        todo!();
    }

    fn parse_hand_meld(&mut self, exp: &str) {
        let mut exp_hand = "".to_string();
        let mut exp_melds = vec![];
        for e in exp.split(',') {
            if exp_hand == "" {
                if e.chars().last().unwrap() == '+' {
                    self.is_drawn = false;
                }
                exp_hand = e.replace("+", "");
            } else {
                exp_melds.push(e.to_string());
            }
        }

        // parse hands
        for t in tiles_from_string(&exp_hand) {
            self.hand[t.0][t.1] += 1;
            if t.1 == 0 {
                self.hand[t.0][5] += 1;
            }
            self.win_tile = t;
        }

        // parse melds
        for exp_meld in &exp_melds {
            self.melds.push(meld_from_string(exp_meld, self.seat));
        }

        if self.is_drawn {
            self.yaku_flags.menzentsumo = true;
            for m in &self.melds {
                if m.type_ != MeldType::Ankan {
                    self.yaku_flags.menzentsumo = false;
                }
            }
        }
    }

    fn parse_yaku_flags(&mut self, exp: &str) {
        todo!();
    }

    fn parse_score_verify(&mut self, exp: &str) {
        todo!();
    }
}

fn tiles_from_string(exp: &str) -> Vec<Tile> {
    let mut tiles = vec![];
    let undef: usize = 255;
    let mut ti = undef;
    for c in exp.chars() {
        match c {
            'm' => ti = 0,
            'p' => ti = 1,
            's' => ti = 2,
            'z' => ti = 3,
            '0'..='9' => {
                if ti == undef {
                    error_exit!("tile number befor tile type");
                }
                let ni = c.to_digit(10).unwrap() as usize;
                tiles.push(Tile(ti, ni));
            }
            _ => {
                error_exit!("invalid char: '{}'", c);
            }
        }
    }
    tiles
}

fn meld_from_string(exp: &str, seat: Seat) -> Meld {
    let undef: usize = 255;
    let mut ti = undef;
    let mut nis = vec![];
    let mut from = 0;
    let mut tiles = vec![];
    let mut froms = vec![];
    for c in exp.chars() {
        match c {
            'm' => ti = 0,
            'p' => ti = 1,
            's' => ti = 2,
            'z' => ti = 3,
            '+' => {
                if froms.is_empty() {
                    error_exit!("invalid '+' suffix");
                }
                let last = froms.len() - 1;
                froms[last] = from % SEAT;
            }
            '0'..='9' => {
                if ti == undef {
                    error_exit!("tile number befor tile type");
                }

                from += 1;
                let ni = c.to_digit(10).unwrap() as usize;
                nis.push(if ni == 0 { 5 } else { ni });
                tiles.push(Tile(ti, ni));
                froms.push(seat);
            }
            _ => {
                error_exit!("invalid char: '{}'", c);
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
        error_exit!("invalid meld: '{}'", exp);
    };

    Meld {
        step: 0,
        seat: seat,
        type_: meld_type,
        tiles: tiles,
        froms: froms,
    }
}
