use std::fmt::Write;
use std::fs::File;
use std::io::{self, BufRead};

use crate::control::common::*;
use crate::hand::{YakuFlags, evaluate_hand};
use crate::model::*;
use crate::util::misc::*;

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

        if !exp.is_empty()
            && let Err(e) = self.process_expression(&exp)
        {
            error!("{}", e);
            return;
        }

        if !file_path.is_empty()
            && let Err(e) = self.run_from_file(&file_path)
        {
            error!("{}", e);
        }
    }

    fn run_from_file(&self, file_path: &str) -> Res {
        let file = File::open(file_path)?;
        let lines = io::BufReader::new(file).lines();
        for exp in lines.map_while(Result::ok) {
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

    fn process_expression(&self, exp: &str) -> Res {
        let mut calculator = Calculator::new(self.detail);
        calculator.parse(exp)?;
        calculator.run();
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

    fn parse(&mut self, input: &str) -> Res {
        println!("> {}", input);

        let input = input.replace(' ', "");
        let input = input.split('#').collect::<Vec<&str>>()[0]; // コメント削除
        let exps: Vec<&str> = input.split('/').collect();
        let len = exps.len();
        if len > 1 {
            self.parse_stage_info(exps[1])?; // 副露のパースに座席情報が必要なので最初に実行
        };
        if len > 0 {
            self.parse_hand_meld(exps[0])?;
        };
        if len > 2 {
            self.parse_yaku_flags(exps[2])?;
        };
        if len > 3 {
            self.parse_score_verify(exps[3])?;
        }

        if self.detail {
            println!("{:?}", self);
        }

        Ok(())
    }

    fn run(&self) -> Verify {
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
            verify
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
            verify
        }
    }

    fn parse_stage_info(&mut self, input: &str) -> Res {
        let exps: Vec<&str> = input.split(',').collect();
        let len = exps.len();
        if len > 0 {
            let chars: Vec<char> = exps[0].chars().collect();
            if chars.len() != 2 {
                Err(format!("stage info len is not 2: {}", exps[0]))?;
            }
            let prevalent_wind = wind_from_char(chars[0])?;
            let seat_wind = wind_from_char(chars[1])?;

            self.prevalent_wind = prevalent_wind;
            self.seat_wind = seat_wind;
            self.is_dealer = seat_wind == 1;
        }
        if len > 1 {
            self.doras = tiles_from_string(exps[1])?;
        }
        if len > 2 {
            self.ura_doras = tiles_from_string(exps[2])?;
        }
        Ok(())
    }

    fn parse_hand_meld(&mut self, input: &str) -> Res {
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

    fn parse_yaku_flags(&mut self, input: &str) -> Res {
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
                _ => Err(format!("invalid conditional yaku: {}", y))?,
            }
        }
        Ok(())
    }

    fn parse_score_verify(&mut self, input: &str) -> Res {
        let exps: Vec<&str> = input.split(',').collect();
        if exps.len() != 3 {
            Err(format!("invalid score verify info: {}", input))?;
        }
        self.fu = exps[0].parse::<usize>()?;
        self.fan = exps[1].parse::<usize>()?;
        self.score = exps[2].parse::<Score>()?;
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
            calculator.parse(&e).unwrap();
            assert_ne!(Verify::Error, calculator.run());
        }
    }
}
