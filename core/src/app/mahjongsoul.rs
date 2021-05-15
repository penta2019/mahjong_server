use serde_json::{json, Value};

use crate::model::*;
use crate::util::common::{as_str, as_usize, unixtime_now};
use crate::util::operator::*;
use crate::util::ws_server::{create_ws_server, SendRecv};

use PlayerOperation::*;

const NO_SEAT: usize = 4;

#[derive(Debug, Default)]
struct Mahjongsoul {
    stage: Stage,
    step: usize,
    seat: usize, // my seat
    actions: Vec<serde_json::Value>,
    need_write: bool,
}

impl Mahjongsoul {
    fn new(need_write: bool) -> Self {
        let mut s = Self::default();
        s.need_write = need_write;
        s
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
        let mut f = std::fs::File::create(file_name).unwrap();
        let arr = serde_json::Value::Array(self.actions.clone());
        write!(f, "{}", serde_json::to_string_pretty(&arr).unwrap()).unwrap();
    }

    fn update_doras(&mut self, data: &Value) {
        let stg = &mut self.stage;
        if let Value::Array(doras) = &data["doras"] {
            if doras.len() > stg.doras.len() {
                let t = tile_from_symbol(as_str(doras.last().unwrap()));
                stg.add_dora(t);
            }
        }
    }

    fn handler_mjstart(&mut self, _data: &Value) {}

    fn handler_newround(&mut self, data: &Value) {
        let mut hand: Vec<Tile> = Vec::new();
        for ps in data["tiles"].as_array().unwrap() {
            hand.push(tile_from_symbol(as_str(ps)));
        }
        let mut player_lands = [vec![], vec![], vec![], vec![]];
        player_lands[self.seat] = hand;

        let mut doras: Vec<Tile> = Vec::new();
        for ps in data["doras"].as_array().unwrap() {
            doras.push(tile_from_symbol(as_str(ps)));
        }

        let round = as_usize(&data["chang"]);
        let hand = as_usize(&data["ju"]);
        let ben = as_usize(&data["ben"]);
        let riichi_sticks = as_usize(&data["liqibang"]);
        let mut scores = [0; SEAT];
        for (s, score) in data["scores"].as_array().unwrap().iter().enumerate() {
            scores[s] = score.as_i64().unwrap() as i32;
        }
        self.stage.op_roundnew(
            round,
            hand,
            ben,
            riichi_sticks,
            &doras,
            &scores,
            &player_lands,
        );
    }

    fn handler_dealtile(&mut self, data: &Value) {
        let stg = &mut self.stage;
        let s = as_usize(&data["seat"]);

        if let Value::String(ps) = &data["tile"] {
            let t = tile_from_symbol(&ps);
            stg.op_dealtile(s, Some(t));
        } else {
            stg.op_dealtile(s, None);
        }

        self.update_doras(data);
    }

    fn handler_discardtile(&mut self, data: &Value) {
        let s = as_usize(&data["seat"]);
        let t = tile_from_symbol(as_str(&data["tile"]));
        let m = data["moqie"].as_bool().unwrap();
        let r = data["is_liqi"].as_bool().unwrap();
        self.stage.op_discardtile(s, t, m, r);
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

        self.stage.op_chiiponkan(s, tp, &tiles, &froms);
    }

    fn handler_angangaddgang(&mut self, data: &Value) {
        let s = as_usize(&data["seat"]);
        let mt = match as_usize(&data["type"]) {
            2 => MeldType::Kakan,
            3 => MeldType::Ankan,
            _ => panic!("invalid gang type"),
        };
        let t = tile_from_symbol(as_str(&data["tiles"]));
        self.stage.op_ankankakan(s, mt, t);
    }

    fn handler_babei(&mut self, data: &Value) {
        let s = as_usize(&data["seat"]);
        let m = data["moqie"].as_bool().unwrap();
        self.stage.op_kita(s, m);
    }

    fn handler_hule(&mut self, data: &Value) {
        // TODO
        let mut delta_scores = [0; 4];
        for (s, score) in data["delta_scores"].as_array().unwrap().iter().enumerate() {
            delta_scores[s] = score.as_i64().unwrap() as i32;
        }
        self.stage.op_roundend_win(&vec![], &vec![], &delta_scores);
        self.write_to_file();
    }

    fn handler_liuju(&mut self, _data: &Value) {
        // TODO
        self.stage.op_roundend_draw(DrawType::Kyushukyuhai);
        self.write_to_file();
    }

    fn handler_notile(&mut self, _data: &Value) {
        // TODO
        // self.stage.op_roundend_notile();
        self.write_to_file();
    }
}

// Application ================================================================

pub struct App {
    game: Mahjongsoul,
    wws_send_recv: SendRecv,
    cws_send_recv: SendRecv,
    file_in: String,
    operator: Box<dyn Operator>,
}

impl App {
    pub fn new(args: Vec<String>) -> Self {
        use std::process::exit;

        // use crate::operator::manual::ManualOperator;
        // use crate::operator::random::RandomDiscardOperator;
        use crate::operator::bot_tiitoitsu::TiitoitsuBot;

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

        Self {
            game: Mahjongsoul::new(need_write),
            wws_send_recv: create_ws_server(52001), // for Web-interface
            cws_send_recv: create_ws_server(52000), // for Controller(mahjongsoul)
            file_in,
            operator: Box::new(TiitoitsuBot::new()),
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
                    println!("operation: {}", dd);

                    if dd["operation_list"] == json!(null) {
                        return;
                    }

                    let stg = &self.game.stage;
                    let seat = dd["seat"].as_i64().unwrap() as Seat;

                    // ゲーム側の演出待ちのため最初の打牌は少し待機
                    if stg.turn == seat && stg.players[seat].discards.len() == 0 {
                        std::thread::sleep(std::time::Duration::from_millis(2000));
                    }

                    let ops = json_parse_operation(dd);
                    println!("ops: {:?}", ops);
                    let op = self.operator.handle_operation(stg, seat, &ops);
                    let (_, arg_idx) = get_operation_index(&ops, &op);
                    match &op {
                        Nop => {
                            if stg.turn == seat {
                                let idx = 13 - stg.players[seat].melds.len() * 3;
                                self.action_dapai(idx);
                            } else {
                                self.action_cancel();
                            }
                        }
                        Discard(v) => {
                            let idx = get_dapai_index(stg, seat, v[0], false);
                            self.action_dapai(idx);
                        }
                        Ankan(_) => {
                            self.action_gang(arg_idx); // TODO
                        }
                        Kakan(_) => {
                            self.action_gang(arg_idx); // TODO
                        }
                        Riichi(v) => {
                            let idx = get_dapai_index(stg, seat, v[0], false);
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
                        Chii(_) => {
                            self.action_chi(arg_idx);
                        }
                        Pon(_) => {
                            self.action_peng(arg_idx);
                        }
                        Minkan(_) => {
                            self.action_gang(arg_idx); // TODO
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

fn get_dapai_index(stage: &Stage, seat: Seat, tile: Tile, is_drawn: bool) -> usize {
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

fn json_parse_operation(v: &Value) -> Vec<PlayerOperation> {
    let mut ops = vec![Nop]; // Nop: ツモ切り or スキップ
    for op in v["operation_list"].as_array().unwrap() {
        match op["type"].as_i64().unwrap() {
            0 => panic!(),
            1 => {
                // 打牌
                let combs = if op["combination"] != json!(null) {
                    json_parse_combination_single(&op["combination"])
                } else {
                    vec![]
                };
                ops.push(Discard(combs));
            }
            2 => {
                // チー
                let combs = json_parse_combination_double(&op["combination"]);
                ops.push(Chii(combs));
            }
            3 => {
                // ポン
                let combs = json_parse_combination_double(&op["combination"]);
                ops.push(Pon(combs));
            }
            4 => {
                // 暗槓
                let combs = json_parse_combination_single(&op["combination"]);
                ops.push(Ankan(combs));
            }
            5 => {
                // 明槓
                let combs = json_parse_combination_single(&op["combination"]);
                ops.push(Minkan(combs));
            }
            6 => {
                // 加槓
                let combs = json_parse_combination_single(&op["combination"]);
                ops.push(Kakan(combs));
            }
            7 => {
                // リーチ
                let combs = json_parse_combination_single(&op["combination"]);
                ops.push(Riichi(combs));
            }
            8 => {
                // ツモ
                ops.push(Tsumo);
            }
            9 => {
                // ロン
                ops.push(Ron);
            }
            10 => {
                // 九種九牌
                ops.push(Kyushukyuhai);
            }
            11 => {
                // 北抜き
                ops.push(Kita);
            }
            _ => panic!(),
        }
    }

    ops
}

fn json_parse_combination_single(v: &Value) -> Vec<Tile> {
    let mut combs = vec![];
    for c in v.as_array().unwrap() {
        combs.push(tile_from_symbol(c.as_str().unwrap()));
    }
    combs
}

fn json_parse_combination_double(v: &Value) -> Vec<(Tile, Tile)> {
    let mut combs = vec![];
    for c_arr in v.as_array().unwrap() {
        let mut comb = vec![];
        for c in c_arr.as_str().unwrap().split('|') {
            comb.push(tile_from_symbol(c));
        }
        combs.push((comb[0], comb[1]));
    }
    combs
}
