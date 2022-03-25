use std::time;

use serde_json::{json, Value};

use crate::actor::create_actor;
use crate::controller::*;
use crate::hand::{get_score_title, Yaku};
use crate::listener::{EventWriter, StageSender};
use crate::model::*;
use crate::util::common::*;
use crate::util::server::Server;

use crate::error;

use ActionType::*;

// [App]
pub struct MahjongsoulApp {
    read_only: bool,
    sleep: bool,
    write: bool,
    write_raw: bool, // mahjongsoul format
    msc_port: u32,
    gui_port: u32,
    actor_name: String,
}

impl MahjongsoulApp {
    pub fn new(args: Vec<String>) -> Self {
        use std::process::exit;

        let mut app = Self {
            read_only: false,
            sleep: false,
            write: false,
            write_raw: false,
            msc_port: super::MSC_PORT,
            gui_port: super::GUI_PORT,
            actor_name: "".to_string(),
        };

        let mut it = args.iter();
        while let Some(s) = it.next() {
            match s.as_str() {
                "-r" => app.read_only = true,
                "-s" => app.sleep = true,
                "-w" => app.write = true,
                "-wr" => app.write_raw = true,
                "-msc-port" => app.msc_port = next_value(&mut it, "-msc-port"),
                "-gui-port" => app.gui_port = next_value(&mut it, "-gui-port"),
                "-0" => app.actor_name = next_value(&mut it, "-0"),
                opt => {
                    error!("unknown option: {}", opt);
                    exit(0);
                }
            }
        }

        app
    }

    pub fn run(&mut self) {
        let actor = create_actor(&self.actor_name);
        println!("actor: {:?}", actor);

        let mut listeners: Vec<Box<dyn Listener>> = vec![];
        let server = Server::new_ws_server(&format!("localhost:{}", self.gui_port));
        listeners.push(Box::new(StageSender::new(server)));
        if self.write {
            listeners.push(Box::new(EventWriter::new()));
            // listeners.push(Box::new(TenhouEventWriter::new(TenhouLog::new())));
        };

        let mut game = Mahjongsoul::new(self.sleep, actor, listeners, self.write_raw);
        let mut server_msc = Server::new_ws_server(&format!("localhost:{}", self.msc_port));
        loop {
            if server_msc.is_new() {
                let msg = r#"{"id": "id_mjaction", "op": "subscribe", "data": "mjaction"}"#;
                server_msc.send(msg.to_string());
            }
            if let Some(msg) = server_msc.recv_timeout(1000) {
                if let Some(act) = game.apply(&serde_json::from_str(&msg).unwrap()) {
                    if !self.read_only {
                        let msg = json!({
                            "id": "0",
                            "op": "eval",
                            "data": act,
                        });
                        server_msc.send(msg.to_string());
                    }
                }
            }
        }
    }
}

// [Mahjongsoul]
#[derive(Debug)]
struct Mahjongsoul {
    ctrl: StageController,
    step: usize,
    seat: usize, // my seat
    events: Vec<Value>,
    random_sleep: bool,
    actor: Box<dyn Actor>,
    write_raw: bool, // 雀魂フォーマットでeventを出力
    start_time: u64,
    round_index: i32,
}

impl Mahjongsoul {
    fn new(
        random_sleep: bool,
        actor: Box<dyn Actor>,
        listeners: Vec<Box<dyn Listener>>,
        write_raw: bool,
    ) -> Self {
        // 新しい局が開始されて座席が判明した際にスワップする
        let nop = create_actor("Nop");
        let actors: [Box<dyn Actor>; SEAT] = [
            nop.clone_box(),
            nop.clone_box(),
            nop.clone_box(),
            nop.clone_box(),
        ];
        Self {
            ctrl: StageController::new(actors, listeners),
            step: 0,
            seat: NO_SEAT,
            events: vec![],
            random_sleep: random_sleep,
            actor: actor,
            write_raw: write_raw,
            start_time: unixtime_now(),
            round_index: 0,
        }
    }

    #[inline]
    fn get_stage(&self) -> &Stage {
        self.ctrl.get_stage()
    }

    #[inline]
    fn handle_event(&mut self, event: Event) {
        self.ctrl.handle_event(&event);

        let mut write = false;
        match event {
            Event::GameStart(_) => {
                self.start_time = unixtime_now();
                self.round_index = 0;
            }
            Event::RoundEndWin(_) | Event::RoundEndDraw(_) => {
                write = true;
            }
            Event::GameOver(_) => {}
            _ => {}
        }

        if self.write_raw && write {
            write_to_file(
                &format!("data_raw/{}/{:2}.json", self.start_time, self.round_index),
                &serde_json::to_string_pretty(&json!(self.events)).unwrap(),
            );
            self.round_index += 1;
        }
    }

    fn apply(&mut self, msg: &Value) -> Option<Value> {
        match as_str(&msg["id"]) {
            "id_mjaction" => {
                if msg["type"] == json!("message") {
                    self.apply_data(&msg["data"], false)
                } else if msg["type"] == json!("message_cache") {
                    self.apply_data(&msg["data"], true)
                } else {
                    None
                }
            }
            _ => None, // type: "success"
        }
    }

    fn apply_data(&mut self, event: &Value, is_cache: bool) -> Option<Value> {
        let step = as_usize(&event["step"]);
        let name = as_str(&event["name"]);
        let data = &event["data"];

        if step == 0 {
            if self.seat != NO_SEAT {
                self.ctrl.swap_actor(self.seat, &mut self.actor);
                self.seat = NO_SEAT;
            }
            self.step = 0;
            self.events.clear();
        }

        self.events.push(event.clone());
        if self.seat == NO_SEAT {
            if let Value::Object(act) = &data["operation"] {
                self.seat = as_usize(&act["seat"]);
            }

            if name == "ActionDealTile" {
                if let Value::String(_) = &data["tile"] {
                    self.seat = as_usize(&data["seat"]);
                }
            }

            if self.seat == NO_SEAT {
                return None;
            }

            // seatが確定し時点でactorを設定
            self.actor.init(self.seat);
            self.ctrl.swap_actor(self.seat, &mut self.actor);
        }

        let mut act = None;
        while self.step < self.events.len() {
            let event = self.events[self.step].clone();
            assert!(self.step == as_usize(&event["step"]));

            let data = &event["data"];
            let name = &event["name"];
            let is_last = self.step + 1 == self.events.len();
            if !is_cache && is_last && as_str(name) == "ActionNewRound" {
                sleep_ms(3000);
            }
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
                s => panic!("unknown event {}", s),
            };
            self.step += 1;

            if !is_cache {
                let a = &data["operation"];
                if a != &json!(null) {
                    // self.ctrl.select_actionはstageを更新した直後sleepを挟まずに実行する必要がある
                    act = self.select_action(a);
                }
            }
        }

        act
    }

    fn select_action(&mut self, data: &Value) -> Option<Value> {
        if data["operation_list"] == json!(null) {
            return None;
        }

        let start = time::Instant::now();
        let s = as_usize(&data["seat"]);

        // 可能なactionのパースと選択
        let (acts, idxs) = json_parse_possible_action(data, self.get_stage());
        let act = self.ctrl.select_action(s, &acts);
        println!("possible: {:?}", acts);
        println!("selected: {:?}", act);
        println!("");
        flush();

        // 選択されたactionのパース
        let arg_idx = if act.0 == Discard || act.0 == Riichi {
            0
        } else {
            idxs[acts.iter().position(|act2| act2 == &act).unwrap()]
        };
        let Action(tp, cs) = act;

        // sleep処理
        let stg = self.get_stage();
        let ellapsed = start.elapsed().as_millis();
        let mut sleep = 1000;
        if self.random_sleep && s == stg.turn && tp != Tsumo {
            // ツモ・ロン・鳴きのキャンセル以外の操作の場合,ランダムにsleep時間(1 ~ 4秒)を取る
            use rand::distributions::{Bernoulli, Distribution};
            let d = Bernoulli::new(0.1).unwrap();
            let mut c = 0;
            loop {
                if c == 30 || d.sample(&mut rand::thread_rng()) {
                    break;
                }
                sleep += 100;
                c += 1;
            }
        }
        if sleep > ellapsed {
            sleep_ms((sleep - ellapsed) as u64);
        }

        let action = match tp {
            Nop => {
                if stg.turn == s {
                    let idx = 13 - stg.players[s].melds.len() * 3;
                    format!("action_dapai({})", idx)
                } else {
                    format!("action_cancel()")
                }
            }
            Discard => {
                let idx = calc_dapai_index(stg, s, cs[0], false);
                format!("action_dapai({})", idx)
            }
            Ankan => {
                format!("action_gang({})", arg_idx)
            }
            Kakan => {
                format!("action_gang({})", arg_idx)
            }
            Riichi => {
                let idx = calc_dapai_index(stg, s, cs[0], false);
                format!("action_lizhi({})", idx)
            }
            Tsumo => {
                format!("action_zimo()")
            }
            Kyushukyuhai => {
                format!("action_jiuzhongjiupai()")
            }
            Kita => {
                format!("action_babei()")
            }
            Chi => {
                format!("action_chi({})", arg_idx)
            }
            Pon => {
                format!("action_peng({})", arg_idx)
            }
            Minkan => {
                format!("action_gang({})", arg_idx)
            }
            Ron => {
                format!("action_hu()")
            }
        };
        Some(json!(format!("msc.ui.{}", action)))
    }

    fn update_doras(&mut self, data: &Value) {
        let stg = self.get_stage();
        if let Value::Array(doras) = &data["doras"] {
            if doras.len() > stg.doras.len() {
                let t = tile_from_mjsoul2(doras.last().unwrap());
                self.handle_event(Event::dora(t));
            }
        }
    }

    fn handler_mjstart(&mut self, _data: &Value) {
        self.handle_event(Event::game_start());
    }

    fn handler_newround(&mut self, data: &Value) {
        let round = as_usize(&data["chang"]);
        let kyoku = as_usize(&data["ju"]);
        let honba = as_usize(&data["ben"]);
        let kyoutaku = as_usize(&data["liqibang"]);
        let mode = as_usize(&data["mode"]);
        let doras = as_vec(tile_from_mjsoul2, &data["doras"]);

        let mut scores = [0; SEAT];
        for (s, score) in as_enumerate(&data["scores"]) {
            scores[s] = as_i32(&score);
        }

        let mut hands = [vec![], vec![], vec![], vec![]];
        for s in 0..SEAT {
            if s == self.seat {
                hands[s] = as_vec(tile_from_mjsoul2, &data["tiles"]);
            } else {
                let hand = &mut hands[s];
                if s == kyoku {
                    // 親番は手牌14枚から開始
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

        self.handle_event(Event::round_new(
            round, kyoku, honba, kyoutaku, doras, scores, hands, mode,
        ));
    }

    fn handler_dealtile(&mut self, data: &Value) {
        self.update_doras(data);
        let s = as_usize(&data["seat"]);

        if let Value::String(ps) = &data["tile"] {
            let t = tile_from_mjsoul(&ps);
            self.handle_event(Event::deal_tile(s, t));
        } else {
            self.handle_event(Event::deal_tile(s, Z8));
        }
    }

    fn handler_discardtile(&mut self, data: &Value) {
        let s = as_usize(&data["seat"]);
        let t = tile_from_mjsoul2(&data["tile"]);
        let m = as_bool(&data["moqie"]);
        let r = as_bool(&data["is_liqi"]);
        self.handle_event(Event::discard_tile(s, t, m, r));
        self.update_doras(data);
    }

    fn handler_chipenggang(&mut self, data: &Value) {
        let s = as_usize(&data["seat"]);
        let tp = match as_usize(&data["type"]) {
            0 => MeldType::Chi,
            1 => MeldType::Pon,
            2 => MeldType::Minkan,
            _ => panic!("unknown meld type"),
        };

        let tiles = as_vec(tile_from_mjsoul2, &data["tiles"]);
        let froms = as_vec(as_usize, &data["froms"]);

        let mut consumed = vec![];
        for (&t, &f) in tiles.iter().zip(froms.iter()) {
            if s == f {
                consumed.push(t);
            }
        }

        self.handle_event(Event::meld(s, tp, consumed));
    }

    fn handler_angangaddgang(&mut self, data: &Value) {
        let s = as_usize(&data["seat"]);
        let tp = match as_usize(&data["type"]) {
            2 => MeldType::Kakan,
            3 => MeldType::Ankan,
            _ => panic!("invalid gang type"),
        };

        let mut t = tile_from_mjsoul2(&data["tiles"]);
        let consumed = if tp == MeldType::Ankan {
            t = t.to_normal();
            let t0 = if t.is_suit() && t.1 == 5 {
                Tile(t.0, 0)
            } else {
                t
            };
            vec![t, t, t, t0] // t0は数牌の5の場合,赤5になる
        } else {
            vec![t]
        };
        self.handle_event(Event::meld(s, tp, consumed));
    }

    fn handler_babei(&mut self, data: &Value) {
        let s = as_usize(&data["seat"]);
        let m = as_bool(&data["moqie"]);

        self.handle_event(Event::kita(s, m));
    }

    fn handler_hule(&mut self, data: &Value) {
        let mut delta_scores = [0; SEAT];
        for (s, score) in as_enumerate(&data["delta_scores"]) {
            delta_scores[s] = as_i32(score);
        }

        let mut wins = vec![];
        for win in as_array(&data["hules"]) {
            let s = as_usize(&win["seat"]);
            let count = as_usize(&win["count"]);
            let is_yakuman = as_bool(&win["yiman"]);
            let fu = as_usize(&win["fu"]);
            let fan = if is_yakuman { 0 } else { count };
            let yakuman_times = if is_yakuman { count } else { 0 };
            let mut yakus = vec![];
            for yaku in as_array(&win["fans"]) {
                let id = as_usize(&yaku["id"]);
                let val = as_usize(&yaku["val"]);
                let stg = self.get_stage();
                let jp_wind = ["?", "東", "南", "西", "北"];
                match id {
                    10 => {
                        // 自風
                        yakus.push((format!("自風 {}", jp_wind[stg.get_seat_wind(s)]), 1));
                    }
                    11 => {
                        // 場風
                        yakus.push((format!("場風 {}", jp_wind[stg.get_prevalent_wind()]), 1));
                    }
                    _ => {
                        if let Some(y) = Yaku::get_from_id(id) {
                            yakus.push((y.name.to_string(), val));
                        } else {
                            error!("yaku not found: id = {}", id);
                        }
                    }
                };
            }
            let ctx = WinContext {
                hand: vec![], // TODO
                yakus: yakus,
                is_tsumo: as_bool(&win["zimo"]),
                score_title: get_score_title(fu, fan, yakuman_times),
                fu: fu,
                fan: fan,
                yakuman_times: yakuman_times,
                points: (
                    as_i32(&win["point_rong"]),
                    as_i32(&win["point_zimo_xian"]),
                    win["point_zimo_qin"].as_i64().unwrap_or(0) as Point,
                ),
            };
            wins.push((s, delta_scores.clone(), ctx));
            delta_scores = [0; SEAT]; // ダブロン,トリロンの場合の内訳は不明なので最初の和了に集約
        }

        self.handle_event(Event::round_end_win(vec![], wins));
    }

    fn handler_liuju(&mut self, data: &Value) {
        let mut draw_type = DrawType::Unknown;
        let mut hands = [vec![], vec![], vec![], vec![]];
        let tenpais = [false; 4];
        let points = [0; 4];
        match as_usize(&data["type"]) {
            1 => {
                // 九種九牌
                draw_type = DrawType::Kyushukyuhai;
                let s = as_usize(&data["seat"]);
                hands[s] = as_vec(tile_from_mjsoul2, &data["tiles"]);
            }
            2 => {
                // 四風連打
                draw_type = DrawType::Suufuurenda;
            }
            3 => {
                // 四槓散了
                draw_type = DrawType::Suukansanra;
            }
            4 => {
                // 四家立直
                draw_type = DrawType::Suuchariichi;
            }
            5 => {
                // 三家和
                draw_type = DrawType::Sanchaho;
            }
            _ => {}
        }

        self.handle_event(Event::round_end_draw(draw_type, hands, tenpais, points));
    }

    fn handler_notile(&mut self, data: &Value) {
        let mut points = [0; SEAT];
        if let Some(ds) = &data["scores"][0]["delta_scores"].as_array() {
            for (s, score) in ds.iter().enumerate() {
                points[s] = as_i32(score);
            }
        }

        let mut tenpais = [false; SEAT];
        for (s, player) in as_enumerate(&data["players"]) {
            tenpais[s] = as_bool(&player["tingpai"]);
        }

        // TODO
        self.handle_event(Event::round_end_draw(
            DrawType::Kouhaiheikyoku,
            [vec![], vec![], vec![], vec![]],
            tenpais,
            points,
        ));
    }
}

fn tile_from_mjsoul(s: &str) -> Tile {
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

fn tile_from_mjsoul2(v: &Value) -> Tile {
    tile_from_mjsoul(as_str(v))
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
                if ti == t.0 && ni == t.to_normal().1 && !is_drawn {
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
                if ti == d.0 && ni == d.to_normal().1 {
                    idx -= 1;
                }
            }
        }
    }

    if !is_drawn {
        error!("tile {} not found", t);
    }

    idx
}

// Actionと元々のデータの各Action内のIndexを返す
fn json_parse_possible_action(v: &Value, stg: &Stage) -> (Vec<Action>, Vec<Index>) {
    let mut acts = vec![Action::nop()]; // Nop: ツモ切り or スキップ
    let mut idxs = vec![0];
    let mut push = |act: Action, idx: usize| {
        acts.push(act);
        idxs.push(idx);
    };

    for act in as_array(&v["operation_list"]) {
        let combs = &act["combination"];
        match as_usize(&act["type"]) {
            0 => panic!(),
            1 => {
                // 打牌
                let combs = if act["combination"] != json!(null) {
                    json_parse_combination(combs)
                } else {
                    vec![vec![]]
                };
                push(Action(Discard, combs[0].clone()), 0);
            }
            2 => {
                // チー
                for (idx, comb) in json_parse_combination(combs).iter().enumerate() {
                    push(Action::chi(comb.clone()), idx);
                }
            }
            3 => {
                // ポン
                for (idx, comb) in json_parse_combination(combs).iter().enumerate() {
                    push(Action::pon(comb.clone()), idx);
                }
            }
            4 => {
                // 暗槓
                for (idx, comb) in json_parse_combination(combs).iter().enumerate() {
                    push(Action::ankan(comb.clone()), idx);
                }
            }
            5 => {
                // 明槓
                for (idx, comb) in json_parse_combination(combs).iter().enumerate() {
                    push(Action::minkan(comb.clone()), idx);
                }
            }
            6 => {
                // 加槓
                // 赤5を含む場合,ponした牌の組み合わせに関係なく combs = ["0p|5p|5p|5p"] となる
                for (idx, comb) in json_parse_combination(combs).iter().enumerate() {
                    let mut t = comb[3];
                    if t.is_suit() && t.1 == 5 && stg.players[stg.turn].hand[t.0][0] > 0 {
                        t = Tile(t.0, 0); // 手牌に赤5があれば通常5を赤5に変換
                    }
                    push(Action::kakan(t), idx);
                }
            }
            7 => {
                // リーチ
                for (idx, comb) in json_parse_combination(combs).iter().enumerate() {
                    push(Action::riichi(comb[0]), idx);
                }
            }
            8 => {
                // ツモ
                push(Action::tsumo(), 0);
            }
            9 => {
                // ロン
                push(Action::ron(), 0);
            }
            10 => {
                // 九種九牌
                push(Action::kyushukyuhai(), 0);
            }
            11 => {
                // 北抜き
                push(Action::kita(), 0);
            }
            _ => panic!(),
        }
    }

    (acts, idxs)
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
            let mut c: Vec<Tile> = comb
                .as_str()
                .unwrap()
                .split('|')
                .map(|sym| tile_from_mjsoul(sym))
                .collect();
            c.sort();
            c
        })
        .collect()
}
