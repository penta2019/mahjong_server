use serde_json::{json, Value};

use crate::model::*;
use crate::util::common::{as_str, as_usize, unixtime_now};
use crate::util::operator::*;
use crate::util::ws_server::{create_ws_server, SendRecv};

use PlayerOperationType::*;

const NO_SEAT: usize = 4;

macro_rules! op {
    ($self:expr, $name:ident $(, $args:expr)*) => {
        paste::item! {
            $self.stage.[< op_ $name>]($($args),*);
            $self.operator.[<notify_op_ $name>](&$self.stage, $($args),*);
        }
    };
}

#[derive(Debug)]
struct Mahjongsoul {
    stage: Stage,
    step: usize,
    seat: usize, // my seat
    actions: Vec<Value>,
    need_write: bool,
    operator: Box<dyn Operator>,
}

impl Mahjongsoul {
    fn new(need_write: bool, operator: Box<dyn Operator>) -> Self {
        Self {
            stage: Stage::default(),
            step: 0,
            seat: 0,
            actions: vec![],
            need_write: need_write,
            operator: operator,
        }
    }

    fn apply(&mut self, act: &Value) {
        let step = as_usize(&act["step"]);
        let name = as_str(&act["name"]);
        let data = &act["data"];

        if step == 0 {
            self.step = 0;
            self.seat = NO_SEAT;
            self.actions.clear();
        }

        self.actions.push(act.clone());
        if self.seat == NO_SEAT {
            if let Value::Object(op) = &data["operation"] {
                self.seat = as_usize(&op["seat"]);
            }

            if name == "ActionDealTile" {
                if let Value::String(_) = &data["tile"] {
                    self.seat = as_usize(&data["seat"]);
                }
            }

            if self.seat == NO_SEAT {
                return;
            }
        }
        self.operator.set_seat(self.seat);

        while self.step < self.actions.len() {
            let action = self.actions[self.step].clone();
            assert!(self.step == as_usize(&action["step"]));

            let data = &action["data"];
            let name = &action["name"];
            match as_str(name) {
                "ActionMJStart" => self.handler_mjstart(data),
                "ActionNewRound" => self.handler_newround(data),
                "ActionDealTile" => self.handler_dealtile(data),
                "ActionDiscardTile" => self.handler_discardtile(data),
                "ActionChiPengGang" => self.handler_chipenggang(data),
                "ActionAnGangAddGang" => self.handler_angangaddgang(data),
                "ActionBabei" => self.handler_babei(data),
                "ActionHule" => self.handler_hule(data),
                "ActionLiuJu" => self.handler_liuju(data),
                "ActionNoTile" => self.handler_notile(data),
                s => panic!("Unknown action {}", s),
            };
            self.step += 1;
        }
    }

    fn write_to_file(&self) {
        use std::io::Write;
        if !self.need_write {
            return;
        }

        let file_name = format!("data/{}.json", unixtime_now().to_string());
        let path = std::path::Path::new(&file_name);
        let prefix = path.parent().unwrap();
        std::fs::create_dir_all(prefix).unwrap();
        let mut f = std::fs::File::create(path).unwrap();
        let arr = Value::Array(self.actions.clone());
        write!(f, "{}", serde_json::to_string_pretty(&arr).unwrap()).unwrap();
    }

    fn update_doras(&mut self, data: &Value) {
        let stg = &mut self.stage;
        if let Value::Array(doras) = &data["doras"] {
            if doras.len() > stg.doras.len() {
                let t = tile_from_symbol(as_str(doras.last().unwrap()));
                op!(self, dora, t);
            }
        }
    }

    fn handler_mjstart(&mut self, _data: &Value) {
        op!(self, game_start);
    }

    fn handler_newround(&mut self, data: &Value) {
        let round = as_usize(&data["chang"]);
        let kyoku = as_usize(&data["ju"]);
        let honba = as_usize(&data["ben"]);
        let kyoutaku = as_usize(&data["liqibang"]);

        let mut doras: Vec<Tile> = Vec::new();
        for ps in data["doras"].as_array().unwrap() {
            doras.push(tile_from_symbol(as_str(ps)));
        }

        let mut scores = [0; SEAT];
        for (s, score) in data["scores"].as_array().unwrap().iter().enumerate() {
            scores[s] = score.as_i64().unwrap() as i32;
        }

        let mut player_hands = [vec![], vec![], vec![], vec![]];
        for s in 0..SEAT {
            let hand = &mut player_hands[s];
            if s == self.seat {
                for ps in data["tiles"].as_array().unwrap() {
                    hand.push(tile_from_symbol(as_str(ps)));
                }
            } else {
                if s == kyoku {
                    for _ in 0..14 {
                        hand.push(Z8);
                    }
                } else {
                    for _ in 0..13 {
                        hand.push(Z8);
                    }
                }
            }
        }

        op!(
            self,
            roundnew,
            round,
            kyoku,
            honba,
            kyoutaku,
            &doras,
            &scores,
            &player_hands
        );
    }

    fn handler_dealtile(&mut self, data: &Value) {
        self.update_doras(data);
        let s = as_usize(&data["seat"]);

        if let Value::String(ps) = &data["tile"] {
            let t = tile_from_symbol(&ps);
            op!(self, dealtile, s, Some(t));
        } else {
            op!(self, dealtile, s, None);
        }
    }

    fn handler_discardtile(&mut self, data: &Value) {
        let s = as_usize(&data["seat"]);
        let t = tile_from_symbol(as_str(&data["tile"]));
        let m = data["moqie"].as_bool().unwrap();
        let r = data["is_liqi"].as_bool().unwrap();
        op!(self, discardtile, s, t, m, r);
        self.update_doras(data);
    }

    fn handler_chipenggang(&mut self, data: &Value) {
        let s = as_usize(&data["seat"]);
        let tp = match as_usize(&data["type"]) {
            0 => MeldType::Chii,
            1 => MeldType::Pon,
            2 => MeldType::Minkan,
            _ => panic!("Unknown meld type"),
        };

        let mut tiles = vec![];
        let mut froms = vec![];
        for ps in data["tiles"].as_array().unwrap() {
            tiles.push(tile_from_symbol(as_str(ps)));
        }
        for f in data["froms"].as_array().unwrap() {
            froms.push(as_usize(f));
        }

        op!(self, chiiponkan, s, tp, &tiles, &froms);
    }

    fn handler_angangaddgang(&mut self, data: &Value) {
        let s = as_usize(&data["seat"]);
        let mt = match as_usize(&data["type"]) {
            2 => MeldType::Kakan,
            3 => MeldType::Ankan,
            _ => panic!("invalid gang type"),
        };
        let t = tile_from_symbol(as_str(&data["tiles"]));
        op!(self, ankankakan, s, mt, t);
    }

    fn handler_babei(&mut self, data: &Value) {
        let s = as_usize(&data["seat"]);
        let m = data["moqie"].as_bool().unwrap();
        op!(self, kita, s, m);
    }

    fn handler_hule(&mut self, data: &Value) {
        // TODO
        let mut score_deltas = [0; 4];
        for (s, score) in data["delta_scores"].as_array().unwrap().iter().enumerate() {
            score_deltas[s] = score.as_i64().unwrap() as i32;
        }
        op!(self, roundend_win, &vec![], &vec![], &score_deltas);
        self.write_to_file();
    }

    fn handler_liuju(&mut self, _data: &Value) {
        // TODO
        op!(self, roundend_draw, DrawType::Kyushukyuhai);
        self.write_to_file();
    }

    fn handler_notile(&mut self, _data: &Value) {
        // TODO
        op!(self, roundend_notile, &[false; SEAT], &[0; SEAT]);
        self.write_to_file();
    }
}

// Application ================================================================

pub struct App {
    game: Mahjongsoul,
    wws_send_recv: SendRecv,
    cws_send_recv: SendRecv,
    file_in: String,
}

impl App {
    pub fn new(args: Vec<String>) -> Self {
        use std::process::exit;

        // use crate::operator::manual::ManualOperator;
        // use crate::operator::random::RandomDiscardOperator;
        use crate::operator::mjai::MjaiEndpoint;
        // use crate::operator::tiitoitsu::TiitoitsuBot;

        let mut file_in = "".to_string();
        let mut need_write = false;
        let mut it = args.iter();
        while let Some(s) = it.next() {
            match s.as_str() {
                "-r" => {
                    if let Some(n) = it.next() {
                        file_in = n.clone();
                    } else {
                        println!("-f: file name missing");
                        exit(0);
                    }
                }
                "-w" => {
                    need_write = true;
                }
                opt => {
                    println!("Unknown option: {}", opt);
                    exit(0);
                }
            }
        }

        let mjai = MjaiEndpoint::new("127.0.0.1:12345");
        Self {
            game: Mahjongsoul::new(need_write, Box::new(mjai)),
            wws_send_recv: create_ws_server(52001), // for Web-interface
            cws_send_recv: create_ws_server(52000), // for Controller(mahjongsoul)
            file_in,
        }
    }

    pub fn run(&mut self) {
        if self.file_in == "" {
            self.read_from_cws();
        } else {
            self.read_from_file();
        }
    }

    fn read_from_file(&mut self) {
        use std::io::{stdin, stdout, Write};

        match std::fs::read_to_string(&self.file_in) {
            Ok(contents) => {
                if let Value::Array(record) = serde_json::from_str(&contents).unwrap() {
                    for v in &record {
                        print!("Enter>");
                        stdout().flush().unwrap();

                        let mut buf = String::new();
                        stdin().read_line(&mut buf).ok();

                        self.game.apply(v);
                        self.game.stage.print();
                        self.send_stage_data();
                    }
                }
                println!("reached end of file: {}", self.file_in);
            }
            Err(err) => {
                println!("IO Error: {}", err);
            }
        }
    }

    fn read_from_cws(&mut self) {
        let mut connected = false;

        loop {
            let msg = if let Some((s, r)) = self.cws_send_recv.lock().unwrap().as_ref() {
                if !connected {
                    connected = true;
                    let msg1 = r#"{"id": "id_mjaction", "op": "subscribe", "data": "mjaction"}"#;
                    let msg2 = r#"{"id": "id_operation", "op": "subscribe", "data": "operation"}"#;
                    s.send(msg1.into()).ok();
                    s.send(msg2.into()).ok();
                }
                match r.recv() {
                    Ok(m) => m,
                    Err(e) => {
                        println!("[Error] {}", e);
                        continue;
                    }
                }
            } else {
                connected = false;
                continue;
            };

            self.handle_cws_msg(&msg);
            self.send_stage_data();
        }
    }

    fn handle_cws_msg(&mut self, msg: &str) {
        let data: Value = serde_json::from_str(&msg).unwrap();
        // println!("data: {}", data);

        match data["id"].as_str().unwrap() {
            "id_mjaction" => {
                let dd = &data["data"];
                if data["type"] == json!("message") {
                    self.game.apply(&dd);
                    // self.game.get_stage().print();
                }
            }
            "id_operation" => {
                let dd = &data["data"];
                if data["type"] == json!("message") {
                    // println!("operation: {}", dd);

                    if dd["operation_list"] == json!(null) {
                        return;
                    }

                    let stg = &self.game.stage;
                    let seat = dd["seat"].as_i64().unwrap() as Seat;

                    // ゲーム側の演出待ちのため最初の打牌は少し待機
                    if stg.turn == seat && stg.players[seat].discards.len() == 0 {
                        std::thread::sleep(std::time::Duration::from_millis(2000));
                    }

                    let (ops, idxs) = json_parse_operation(dd);
                    // println!("ops: {:?}", ops);
                    let op = self.game.operator.handle_operation(stg, seat, &ops);
                    let arg_idx = if op.0 == Discard {
                        0
                    } else {
                        idxs[ops.iter().position(|op2| op2 == &op).unwrap()]
                    };
                    let PlayerOperation(tp, cs) = op;
                    match tp {
                        Nop => {
                            if stg.turn == seat {
                                let idx = 13 - stg.players[seat].melds.len() * 3;
                                self.action_dapai(idx);
                            } else {
                                self.action_cancel();
                            }
                        }
                        Discard => {
                            let idx = calc_dapai_index(stg, seat, cs[0], false);
                            self.action_dapai(idx);
                        }
                        Ankan => {
                            self.action_gang(arg_idx);
                        }
                        Kakan => {
                            self.action_gang(arg_idx);
                        }
                        Riichi => {
                            let idx = calc_dapai_index(stg, seat, cs[0], false);
                            self.action_lizhi(idx);
                        }
                        Tsumo => {
                            self.action_zimo();
                        }
                        Kyushukyuhai => {
                            self.action_jiuzhongjiupai();
                        }
                        Kita => {
                            self.action_babei();
                        }
                        Chii => {
                            self.action_chi(arg_idx);
                        }
                        Pon => {
                            self.action_peng(arg_idx);
                        }
                        Minkan => {
                            self.action_gang(arg_idx);
                        }
                        Ron => {
                            self.action_hu();
                        }
                    }
                }
            }
            _ => {
                // type: "success"
            }
        }
    }

    fn send_stage_data(&mut self) {
        self.send_to_wws("stage", &json!(&self.game.stage));
    }

    // スキップ
    fn action_cancel(&mut self) {
        self.send_action(&format!("action_cancel()"));
    }

    // 打牌
    fn action_dapai(&mut self, idx: Index) {
        self.send_action(&format!("action_dapai({})", idx));
    }

    // チー
    fn action_chi(&mut self, idx: Index) {
        self.send_action(&format!("action_chi({})", idx));
    }

    // ポン
    fn action_peng(&mut self, idx: Index) {
        self.send_action(&format!("action_peng({})", idx));
    }

    // 槓 (暗槓, 明槓, 加槓)
    fn action_gang(&mut self, idx: Index) {
        self.send_action(&format!("action_gang({})", idx));
    }

    // リーチ
    fn action_lizhi(&mut self, idx: Index) {
        self.send_action(&format!("action_lizhi({})", idx));
    }

    // ツモ
    fn action_zimo(&mut self) {
        self.send_action(&format!("action_zimo()"));
    }

    // ロン
    fn action_hu(&mut self) {
        self.send_action(&format!("action_hu()"));
    }

    // 九種九牌
    fn action_jiuzhongjiupai(&mut self) {
        self.send_action(&format!("action_jiuzhongjiupai()"));
    }

    // 北抜き
    fn action_babei(&mut self) {
        self.send_action(&format!("action_babei()"));
    }

    fn send_action(&mut self, func: &str) {
        self.send_to_cws("0", "eval", &json!(format!("msc.ui.{}", func)));
    }

    fn send_to_cws(&mut self, id: &str, op: &str, data: &Value) {
        if let Some((s, _)) = self.cws_send_recv.lock().unwrap().as_ref() {
            let msg = json!({
                "id": id,
                "op": op,
                "data": data,
            });
            s.send(msg.to_string()).ok();
        }
    }

    fn send_to_wws(&mut self, type_: &str, data: &Value) {
        if let Some((s, _)) = self.wws_send_recv.lock().unwrap().as_ref() {
            let msg = json!({
                "type": type_,
                "data": data,
            });
            s.send(msg.to_string()).ok();
        }
    }
}

// Utility ====================================================================

fn tile_from_symbol(s: &str) -> Tile {
    let b = s.as_bytes();
    let n = b[0] - b'0';
    let t = match b[1] as char {
        'm' => 0,
        'p' => 1,
        's' => 2,
        'z' => 3,
        _ => panic!("invalid Tile type"),
    };
    Tile(t, n as usize)
}

fn calc_dapai_index(stage: &Stage, seat: Seat, tile: Tile, is_drawn: bool) -> usize {
    let pl = &stage.players[seat];
    let h = &pl.hand;
    let t = tile;
    let d = if let Some(d) = pl.drawn { d } else { Z8 };
    let is_drawn = if pl.drawn == Some(t) {
        if pl.hand[t.0][t.1] == 1 || (t.1 == 5 && pl.hand[t.0][5] == 2 && pl.hand[t.0][0] == 1) {
            true
        } else {
            is_drawn
        }
    } else {
        if t.1 == 5 && pl.hand[t.0][t.1] == 1 && Some(Tile(t.0, 0)) == pl.drawn {
            true // ツモった赤5を通常5で指定する場合に通常5がなければ赤5をツモ切り
        } else {
            false
        }
    };

    let mut idx = 0;
    for ti in 0..TYPE {
        for ni in 1..TNUM {
            if h[ti][ni] > 0 {
                if ti == t.0 && ni == t.n() && !is_drawn {
                    if ni == 5
                        && h[ti][5] > 1
                        && h[ti][0] == 1
                        && t.1 == 5
                        && pl.drawn != Some(Tile(ti, 0))
                    {
                        return idx + 1; // 赤5が存在しているが指定された牌が通常5の場合
                    } else {
                        return idx;
                    }
                }
                idx += h[ti][ni];
                if ti == d.0 && ni == d.n() {
                    idx -= 1;
                }
            }
        }
    }

    if !is_drawn {
        println!("[Error] Tile {} not found", t);
    }

    idx
}

// PlayerOperationと元々のデータの各Operation内のIndexを返す
fn json_parse_operation(v: &Value) -> (Vec<PlayerOperation>, Vec<Index>) {
    let mut ops = vec![Op::nop()]; // Nop: ツモ切り or スキップ
    let mut idxs = vec![0];
    let mut push = |op: PlayerOperation, idx: usize| {
        ops.push(op);
        idxs.push(idx);
    };

    for op in v["operation_list"].as_array().unwrap() {
        let combs = &op["combination"];
        match op["type"].as_i64().unwrap() {
            0 => panic!(),
            1 => {
                // 打牌
                let combs = if op["combination"] != json!(null) {
                    json_parse_combination(combs)
                } else {
                    vec![vec![]]
                };
                push(PlayerOperation(Discard, combs[0].clone()), 0);
            }
            2 => {
                // チー
                for (idx, comb) in json_parse_combination(combs).iter().enumerate() {
                    push(Op::chii(comb.clone()), idx);
                }
            }
            3 => {
                // ポン
                for (idx, comb) in json_parse_combination(combs).iter().enumerate() {
                    push(Op::pon(comb.clone()), idx);
                }
            }
            4 => {
                // 暗槓
                for (idx, comb) in json_parse_combination(combs).iter().enumerate() {
                    push(Op::ankan(comb.clone()), idx);
                }
            }
            5 => {
                // 明槓
                for (idx, comb) in json_parse_combination(combs).iter().enumerate() {
                    push(Op::minkan(comb.clone()), idx);
                }
            }
            6 => {
                // 加槓
                for (idx, comb) in json_parse_combination(combs).iter().enumerate() {
                    push(Op::kakan(comb[0]), idx);
                }
            }
            7 => {
                // リーチ
                for (idx, comb) in json_parse_combination(combs).iter().enumerate() {
                    push(Op::riichi(comb[0]), idx);
                }
            }
            8 => {
                // ツモ
                push(Op::tsumo(), 0);
            }
            9 => {
                // ロン
                push(Op::ron(), 0);
            }
            10 => {
                // 九種九牌
                push(Op::kyushukyuhai(), 0);
            }
            11 => {
                // 北抜き
                push(Op::kita(), 0);
            }
            _ => panic!(),
        }
    }

    (ops, idxs)
}

fn json_parse_combination(combs: &Value) -> Vec<Vec<Tile>> {
    // combsは以下のようなjson list
    // [
    //     "4s|6s",
    //     "6s|7s"
    // ]
    combs
        .as_array()
        .unwrap()
        .iter()
        .map(|comb| {
            comb.as_str()
                .unwrap()
                .split('|')
                .map(|sym| tile_from_symbol(sym))
                .collect()
        })
        .collect()
}
