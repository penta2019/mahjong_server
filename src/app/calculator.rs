use std::fmt::Write as _;
use std::fs::File;
use std::io::{self, BufRead};

use crate::etc::misc::*;
use crate::hand::{evaluate_hand, YakuFlags};
use crate::model::*;
use crate::util::common::*;

use crate::error;

#[derive(Debug)]
pub struct CalculatorApp {
    args: Vec<String>,
    detail: bool,
}

impl CalculatorApp {
    pub fn new(args: Vec<String>) -> Self {
        Self {
            args,
            detail: false,
        }
    }

    pub fn run(&mut self) {
        let mut file_path = "".to_string();
        let mut exp = "".to_string();
        let mut it = self.args.iter();
        while let Some(s) = it.next() {
            match s.as_str() {
                "-d" => self.detail = true,
                "-f" => file_path = next_value(&mut it, s),
                _ => {
                    if exp.starts_with('-') {
                        error!("unknown option: {}", s);
                        return;
                    }
                    if !exp.is_empty() {
                        error!("multiple expression is not allowed");
                        return;
                    }
                    exp = s.clone();
                }
            }
        }

        if (file_path.is_empty() && exp.is_empty()) || (!file_path.is_empty() && !exp.is_empty()) {
            print_usage();
            return;
        }

        if !exp.is_empty() {
            if let Err(e) = self.process_expression(&exp) {
                error!("{}", e);
                return;
            }
        }

        if !file_path.is_empty() {
            if let Err(e) = self.run_from_file(&file_path) {
                error!("{}", e);
            }
        }
    }

    fn run_from_file(&self, file_path: &str) -> std::io::Result<()> {
        let file = File::open(file_path)?;
        let lines = io::BufReader::new(file).lines();
        for exp in lines.flatten() {
            let e = exp.replace(' ', "");
            if e.is_empty() || e.starts_with('#') {
                // 空行とコメント行はスキップ
                println!("> {}", exp);
            } else if let Err(e) = self.process_expression(&exp) {
                error!("{}", e);
            }
            println!();
        }
        Ok(())
    }

    fn process_expression(&self, exp: &str) -> Result<(), String> {
        let mut calculator = Calculator::new(self.detail);
        calculator.parse(exp)?;
        calculator.run()?;
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
enum Verify {
    Ok,
    Error,
    Skip,
}

#[derive(Debug)]
struct Calculator {
    detail: bool,
    // evaluate_hand params
    hand: TileTable,
    melds: Vec<Meld>,
    doras: Vec<Tile>,
    ura_doras: Vec<Tile>,
    winning_tile: Tile,
    is_drawn: bool,
    is_dealer: bool,
    prevalent_wind: Index,
    seat_wind: Index,
    yaku_flags: YakuFlags,
    // score verify
    verify: bool,
    fu: usize,
    fan: usize,
    score: Score,
}

impl Calculator {
    fn new(detail: bool) -> Self {
        Self {
            detail,
            hand: TileTable::default(),
            melds: vec![],
            doras: vec![],
            ura_doras: vec![],
            winning_tile: Z8,
            is_drawn: true,
            is_dealer: true,
            prevalent_wind: 1,
            seat_wind: 1,
            yaku_flags: YakuFlags::default(),
            verify: false,
            fu: 0,
            fan: 0,
            score: 0,
        }
    }

    fn parse(&mut self, input: &str) -> Result<(), String> {
        println!("> {}", input);

        let input = input.replace(' ', "");
        let input = input.split('#').collect::<Vec<&str>>()[0]; // コメント削除
        let exps: Vec<&str> = input.split('/').collect();

        if let Some(exp) = exps.get(1) {
            self.parse_stage_info(exp)?; // 副露のパースに座席情報が必要なので最初に実行
        };
        if let Some(exp) = exps.get(0) {
            self.parse_hand_meld(exp)?;
        };
        if let Some(exp) = exps.get(2) {
            self.parse_yaku_flags(exp)?;
        };
        if let Some(exp) = exps.get(3) {
            self.parse_score_verify(exp)?;
        }

        if self.detail {
            println!("{:?}", self);
        }

        Ok(())
    }

    fn run(&self) -> Result<Verify, String> {
        if let Some(ctx) = evaluate_hand(
            &self.hand,
            &self.melds,
            &self.doras,
            &self.ura_doras,
            self.winning_tile,
            self.is_drawn,
            self.is_dealer,
            self.prevalent_wind,
            self.seat_wind,
            &self.yaku_flags,
        ) {
            if self.detail {
                println!("{:?}", ctx);
            }

            let mut yakus = "".to_string();
            for y in ctx.yakus {
                let _ = write!(yakus, "{}({}), ", y.name, y.fan);
            }
            println!("yakus: {}", yakus);

            println!(
                "fu: {}, fan: {}, yakuman: {}, score: {}, {}",
                ctx.fu, ctx.fan, ctx.yakuman, ctx.score, ctx.title
            );

            let verify = if self.verify {
                if ctx.yakuman > 0 {
                    // 役満以上は得点のみをチェック
                    if ctx.score == self.score {
                        Verify::Ok
                    } else {
                        Verify::Error
                    }
                } else {
                    if ctx.fu == self.fu && ctx.fan == self.fan && ctx.score == self.score {
                        Verify::Ok
                    } else {
                        Verify::Error
                    }
                }
            } else {
                Verify::Skip
            };
            println!("verify: {:?}", verify);
            Ok(verify)
        } else {
            println!("not win hand");
            let verify = if self.verify {
                if self.score == 0 {
                    Verify::Ok
                } else {
                    Verify::Error
                }
            } else {
                Verify::Skip
            };
            println!("verify: {:?}", verify);
            Ok(verify)
        }
    }

    fn parse_stage_info(&mut self, input: &str) -> Result<(), String> {
        let exps: Vec<&str> = input.split(',').collect();
        if let Some(exp) = exps.get(0) {
            let chars: Vec<char> = exp.chars().collect();
            if chars.len() != 2 {
                return Err(format!("stage info len is not 2: {}", exp));
            }
            let prevalent_wind = wind_from_char(chars[0])?;
            let seat_wind = wind_from_char(chars[1])?;

            self.prevalent_wind = prevalent_wind;
            self.seat_wind = seat_wind;
            self.is_dealer = seat_wind == 1;
        }
        if let Some(exp) = exps.get(1) {
            self.doras = tiles_from_string(exp)?;
        }
        if let Some(exp) = exps.get(2) {
            self.ura_doras = tiles_from_string(exp)?;
        }
        Ok(())
    }

    fn parse_hand_meld(&mut self, input: &str) -> Result<(), String> {
        let mut exp_hand = "".to_string();
        let mut exp_melds = vec![];
        for exp in input.split(',') {
            if exp_hand.is_empty() {
                if exp.ends_with('+') {
                    self.is_drawn = false;
                }
                exp_hand = exp.replace('+', "");
            } else {
                exp_melds.push(exp.to_string());
            }
        }

        // parse hands
        for t in tiles_from_string(&exp_hand)? {
            self.hand[t.0][t.1] += 1;
            if t.1 == 0 {
                self.hand[t.0][5] += 1;
            }
            self.winning_tile = t;
        }

        // parse melds
        for exp_meld in &exp_melds {
            self.melds.push(meld_from_string(exp_meld)?);
        }
        self.yaku_flags.menzentsumo =
            self.is_drawn && self.melds.iter().all(|m| m.meld_type == MeldType::Ankan);

        Ok(())
    }

    fn parse_yaku_flags(&mut self, input: &str) -> Result<(), String> {
        for y in input.split(',') {
            match y {
                "立直" => self.yaku_flags.riichi = true,
                "両立直" => self.yaku_flags.dabururiichi = true,
                "一発" => self.yaku_flags.ippatsu = true,
                "海底摸月" => self.yaku_flags.haiteiraoyue = true,
                "河底撈魚" => self.yaku_flags.houteiraoyui = true,
                "嶺上開花" => self.yaku_flags.rinshankaihou = true,
                "槍槓" => self.yaku_flags.chankan = true,
                "天和" => self.yaku_flags.tenhou = true,
                "地和" => self.yaku_flags.tiihou = true,
                "" => {}
                _ => return Err(format!("invalid conditional yaku: {}", y)),
            }
        }
        Ok(())
    }

    fn parse_score_verify(&mut self, input: &str) -> Result<(), String> {
        let exps: Vec<&str> = input.split(',').collect();
        if exps.len() != 3 {
            return Err(format!("invalid score verify info: {}", input));
        }
        self.fu = exps[0].parse::<usize>().map_err(|e| e.to_string())?;
        self.fan = exps[1].parse::<usize>().map_err(|e| e.to_string())?;
        self.score = exps[2].parse::<Score>().map_err(|e| e.to_string())?;
        self.verify = true;
        Ok(())
    }
}

fn print_usage() {
    error!(
        r"invalid input
Usage
    $ cargo run C EXPRESSION [-d]
    $ cargo run C -f FILE [-d]
Options
    -d: print debug info
    -f: read expresisons from file instead of a commandline expression
"
    );
}

#[test]
fn test_calculator() {
    let file = File::open("tests/win_hands.txt").unwrap();
    let lines = io::BufReader::new(file).lines();
    for exp in lines.flatten() {
        let e = exp.replace(' ', "");
        if e.is_empty() || e.starts_with('#') {
            // 空行とコメント行はスキップ
            println!("> {}", exp);
        } else {
            let mut calculator = Calculator::new(false);
            assert_eq!(Ok(()), calculator.parse(&e));
            assert_ne!(Verify::Error, calculator.run().unwrap());
        }
    }
}
