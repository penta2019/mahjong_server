use std::{
    fmt::Write,
    fs::File,
    io::{self, BufRead},
};

use crate::{control::common::*, error, hand::*, model::*, util::misc::*};

const INDENT: usize = 2;

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
        let mut file_path = String::new();
        let mut exp = String::new();
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
            && let Err(err) = self.process_expression(&exp)
        {
            error!("{}", err);
            return;
        }

        if !file_path.is_empty()
            && let Err(err) = self.run_from_file(&file_path)
        {
            error!("{}", err);
        }
    }

    fn run_from_file(&self, file_path: &str) -> Res {
        let file = File::open(file_path)?;
        let lines = io::BufReader::new(file).lines();
        for exp in lines.map_while(Result::ok) {
            let exp2 = exp.replace(' ', "");
            if exp2.is_empty() || exp2.starts_with('#') {
                // 空行とコメント行はスキップ
                println!("> {}", exp);
            } else if let Err(err) = self.process_expression(&exp) {
                error!("{}", err);
            }
            println!();
        }
        Ok(())
    }

    fn process_expression(&self, exp: &str) -> Res {
        let mut calculator = Calculator::new(self.detail);
        calculator.parse(exp)?;
        calculator.calculate();
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
            println!("{:#?}", self);
            println!();
        }

        Ok(())
    }

    fn calculate(&mut self) -> Verify {
        // 手牌の枚数を集計 (row[0]は赤5のフラグなのでスキップ)
        let tile_num = self
            .hand
            .iter()
            .map(|row| row[1..].iter().sum::<usize>())
            .sum::<usize>();

        let verify = match tile_num % 3 {
            0 => {
                // 麻雀のルール上,手牌の枚数が3の倍数になることはない
                println!("手牌の枚数が不正です");
                None
            }
            1 => {
                // あと1枚(ツモまたはロン)で完成形 -> 和了牌計算
                self.calculate_winnig_tile(0)
            }
            2 => {
                // n面子+雀頭の完成形
                self.calculate_win(0) // 和了形チェック -> 和了点計算
                    .or_else(|| self.calculate_tenpai()) // 聴牌形チェック -> 聴牌打牌計算
            }
            _ => panic!(),
        }
        .unwrap_or_else(|| self.verify_no_socre());

        println!("verify: {verify:?}");
        verify
    }

    fn verify_no_socre(&self) -> Verify {
        if self.verify {
            if self.score == 0 {
                Verify::Ok
            } else {
                Verify::Error
            }
        } else {
            Verify::Skip
        }
    }

    fn calculate_win(&self, indent_size: usize) -> Option<Verify> {
        let Some((sctx, yctx)) = evaluate_hand(
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
        ) else {
            return if is_normal_win(&self.hand) {
                println!("役無し");
                Some(self.verify_no_socre())
            } else {
                None
            };
        };

        let indent = " ".repeat(indent_size);
        // println!("{indent}[和了点計算]");

        if self.detail {
            println!("{:#?}", sctx);
            println!();
            println!("{:#?}", yctx);
            println!();
        }

        println!("{indent}和了牌: {}", self.winning_tile);

        // 面子
        // 面子の配列を生成
        let mut sets: Vec<_> = yctx
            .parsed_hand()
            .clone()
            .into_iter()
            .map(|s| (s.normal_tiles(), s.0))
            .collect();
        // 辞書順で並び替える (手牌と副露は後で振り分けるので混ざってもOK)
        sets.sort_by(|s0, s1| s0.0.cmp(&s1.0));
        let mut hand_str = String::new();
        let mut meld_str = String::new();
        for set in sets {
            let target = match set.1 {
                SetPairType::Pair | SetPairType::Shuntsu | SetPairType::Koutsu => &mut hand_str,
                _ => &mut meld_str,
            };
            if !target.is_empty() {
                write!(target, ", ").ok();
            }
            write!(target, "{}", set.0[0]).ok();
            for t in &set.0[1..] {
                write!(target, "{}", t.1).ok();
            }
        }
        print!("{indent}面子　: {}", hand_str);
        if !meld_str.is_empty() {
            print!(", ({})", meld_str);
        }
        println!();

        // 符計算
        let mut fus_str = String::new();
        for fu in yctx.calc_fu() {
            if !fus_str.is_empty() {
                write!(fus_str, ", ").ok();
            }
            write!(fus_str, "{}[{}]", fu.1, fu.0).ok();
        }
        println!("{indent}符計算: {fus_str}");

        // 役一覧
        let mut yakus_str = String::new();
        for y in sctx.yakus {
            if !yakus_str.is_empty() {
                write!(yakus_str, ", ").ok();
            }
            write!(yakus_str, "{}[{}]", y.name, y.fan).ok();
        }
        println!("{indent}役一覧: {yakus_str}");

        // 和了
        print!(
            "{indent}和了　: {} {} ",
            if self.is_dealer { "親" } else { "子" },
            if self.is_drawn { "ツモ" } else { "ロン" }
        );
        let score_breakdown = if self.is_drawn {
            if self.is_dealer {
                format!("({}オール)", sctx.points.1)
            } else {
                format!("({}/{})", sctx.points.1, sctx.points.2)
            }
        } else {
            String::new()
        };
        if sctx.yakuman == 0 {
            println!(
                "{indent}{}符{}飜 {}{} {}",
                sctx.fu, sctx.fan, sctx.score, score_breakdown, sctx.title
            );
        } else {
            println!("{}{} {}", sctx.score, score_breakdown, sctx.title);
        }

        let verify = if self.verify {
            if sctx.yakuman > 0 {
                // 役満以上は得点のみをチェック
                if sctx.score == self.score {
                    Verify::Ok
                } else {
                    Verify::Error
                }
            } else {
                if sctx.fu == self.fu && sctx.fan == self.fan && sctx.score == self.score {
                    Verify::Ok
                } else {
                    Verify::Error
                }
            }
        } else {
            Verify::Skip
        };
        Some(verify)
    }

    fn calculate_winnig_tile(&mut self, indent_size: usize) -> Option<Verify> {
        let indent = " ".repeat(indent_size);
        // println!("{indent}[和了牌計算]");

        let tiles = calc_tiles_to_win(&self.hand);

        if tiles.is_empty() {
            println!("{indent}聴牌形ではありません");
            return None;
        }

        // print!("{indent}和了牌: {}", tiles[0]);
        // for t in &tiles[1..] {
        //     print!(", {}", t);
        // }
        // println!();

        for tile in tiles {
            inc_tile(&mut self.hand, tile);
            self.winning_tile = tile;
            self.calculate_win(indent_size);
            dec_tile(&mut self.hand, tile);
            println!();
        }

        None
    }

    fn calculate_tenpai(&mut self) -> Option<Verify> {
        // println!("[聴牌打牌計算]");

        let discards = calc_discards_to_win(&self.hand);

        if discards.is_empty() {
            println!("一向聴以上の手牌です");
            return None;
        }

        println!("打牌 => 和了牌");
        for (discard, tiles) in &discards {
            print!("{discard} => {}", tiles[0]);
            for tile in &tiles[1..] {
                print!(", {}", tile);
            }
            println!();
        }
        println!();

        for (discard, _) in &discards {
            println!("打牌: {discard}");
            dec_tile(&mut self.hand, *discard);
            self.calculate_winnig_tile(INDENT);
            inc_tile(&mut self.hand, *discard);
        }

        None
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
        let mut exp_hand = String::new();
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
    -f: read expresisons from file
"
    );
}

#[test]
fn test_calculator() {
    let file = File::open("tests/win_hands.txt").unwrap();
    let lines = io::BufReader::new(file).lines();
    for exp in lines.flatten() {
        let exp2 = exp.replace(' ', "");
        if exp2.is_empty() || exp2.starts_with('#') {
            // 空行とコメント行はスキップ
            println!("> {}", exp);
        } else {
            let mut calculator = Calculator::new(false);
            calculator.parse(&exp2).unwrap();
            assert_ne!(Verify::Error, calculator.calculate());
        }
    }
}
