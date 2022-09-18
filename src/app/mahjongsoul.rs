use std::time;

use serde_json::{json, Value};

use crate::actor::create_actor;
use crate::controller::*;
use crate::hand::{get_score_title, YakuDefine};
use crate::listener::EventWriter;
use crate::model::*;
use crate::util::common::*;
use crate::util::connection::*;

use crate::error;

use ActionType::*;

// [App]
#[derive(Debug)]
pub struct MahjongsoulApp {
    read_only: bool,
    sleep: bool,
    write: bool,
    write_raw: bool, // mahjongsoul format
    msc_port: u32,
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
        if self.write {
            listeners.push(Box::new(EventWriter::new()));
        };
        // Debug port
        let conn = TcpConnection::new("127.0.0.1:52999");
        listeners.push(Box::new(crate::listener::EventSender::new(Box::new(conn))));

        let mut game = Mahjongsoul::new(self.sleep, actor, listeners, self.write_raw);
        let mut conn_msc = WsConnection::new(&format!("127.0.0.1:{}", self.msc_port));
        let mut last_event = None;
        let mut last_event_ts = 0.0;
        loop {
            match conn_msc.recv() {
                Message::Open => {
                    let msg = r#"{"id": "id_mjaction", "op": "subscribe", "data": "mjaction"}"#;
                    conn_msc.send(msg);
                }
                Message::Text(t) => {
                    let msg: &Value = &serde_json::from_str(&t).unwrap();
                    match as_str(&msg["id"]) {
                        "id_mjaction" => {
                            if msg["type"] == json!("message") {
                                // キャッシュデータの場合すぐに次のデータが届く
                                if let Some(e) = last_event {
                                    game.apply(&e);
                                }
                                last_event = Some(msg["data"].clone());
                                last_event_ts = unixtime_now();
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }

            // 0.1秒以上次のデータが来なかった時,リアルタイムのデータとみなす
            if unixtime_now() - last_event_ts > 0.1 {
                if let Some(e) = last_event {
                    game.apply(&e);
                    last_event = None;
                }
                if last_event.is_none() && !self.read_only {
                    if let Some(a) = game.poll_action() {
                        let msg = json!({
                            "id": "0",
                            "op": "eval",
                            "data": a,
                        });
                        conn_msc.send(&msg.to_string());
                    }
                }
            }

            sleep_ms(10);
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
    kyoku_index: i32,
    acts: Vec<Action>,
    acts_idxs: Vec<usize>,
    retry: i32,
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
            random_sleep,
            actor,
            write_raw,
            start_time: unixtime_now() as u64,
            kyoku_index: 0,
            acts: vec![],
            acts_idxs: vec![],
            retry: 0,
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
            Event::Begin(_) => {
                self.start_time = unixtime_now() as u64;
                self.kyoku_index = 0;
            }
            Event::Win(_) | Event::Draw(_) => {
                write = true;
            }
            Event::End(_) => {}
            _ => {}
        }

        if self.write_raw && write {
            write_to_file(
                &format!("data_raw/{}/{:2}.json", self.start_time, self.kyoku_index),
                &serde_json::to_string_pretty(&json!(self.events)).unwrap(),
            );
            self.kyoku_index += 1;
        }
    }

    fn apply(&mut self, event: &Value) {
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
            // operation の含まれているイベントデータが届くまで自身の座席は不明
            if let Value::Object(act) = &data["operation"] {
                self.seat = as_usize(&act["seat"]);
            }

            if name == "ActionDealTile" {
                if let Value::String(_) = &data["tile"] {
                    self.seat = as_usize(&data["seat"]);
                }
            }

            if self.seat == NO_SEAT {
                return;
            }

            // seatが確定し時点でactorを設定
            self.actor.init(self.seat); // TODO: これがないとMjaiが不正なseatを送信ことがある
            self.ctrl.swap_actor(self.seat, &mut self.actor);
        }

        // seatが確定するまでは実行されないので溜まったeventをまとめて処理
        while self.step < self.events.len() {
            let event = self.events[self.step].clone();
            assert!(self.step == as_usize(&event["step"]));

            let data = &event["data"];
            let name = &event["name"];
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

            let acts = data["operation"].clone();
            if acts != json!(null) && acts["operation_list"] != json!(null) {
                (self.acts, self.acts_idxs) = parse_possible_action(&acts, self.get_stage());
                self.retry = 0;
            } else {
                self.acts = vec![];
            }

            self.step += 1;
        }
    }

    fn poll_action(&mut self) -> Option<Value> {
        if self.acts == vec![] {
            return None;
        }

        let start = time::Instant::now();
        let act = self.ctrl.select_action(self.seat, &self.acts, self.retry);
        self.retry += 1;

        act.as_ref()?; // None なら return
        let act = act.unwrap();

        println!("possible: {:?}", self.acts);
        println!("selected: {:?}", act);
        println!();
        flush();

        // 選択されたactionのパース
        let arg_idx = match act.action_type {
            Discard | Riichi => 0,
            _ => self.acts_idxs[self.acts.iter().position(|act2| act2 == &act).unwrap()],
        };

        let tp = act.action_type;
        let cs = &act.tiles;

        // sleep処理
        if self.step <= 1 {
            // ActionNewRoundの直後はゲームの初期化が終わっていない場合があるので長めに待機
            sleep_ms(3000);
        }
        let stg = self.get_stage();
        let ellapsed = start.elapsed().as_millis();
        let mut sleep = 1000;
        if self.random_sleep && self.seat == stg.turn && tp != Tsumo {
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
                if stg.turn == self.seat {
                    let idx = 13 - stg.players[self.seat].melds.len() * 3;
                    format!("action_dapai({})", idx)
                } else {
                    format!("action_cancel()")
                }
            }
            Discard => {
                let idx = calc_dapai_index(stg, self.seat, cs[0], false);
                format!("action_dapai({})", idx)
            }
            Ankan => {
                format!("action_gang({})", arg_idx)
            }
            Kakan => {
                format!("action_gang({})", arg_idx)
            }
            Riichi => {
                let (t, m) = if cs.is_empty() {
                    (stg.players[stg.turn].drawn.unwrap(), true)
                } else {
                    (cs[0], false)
                };
                let idx = calc_dapai_index(stg, self.seat, t, m);
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

        self.acts = vec![];
        Some(json!(format!("msc.ui.{}", action)))
    }

    fn update_doras(&mut self, data: &Value) {
        let stg = self.get_stage();
        if let Value::Array(doras) = &data["doras"] {
            if doras.len() > stg.doras.len() {
                let t = tile_from_mjsoul(doras.last().unwrap());
                self.handle_event(Event::dora(t));
            }
        }
    }

    fn handler_mjstart(&mut self, _data: &Value) {
        self.handle_event(Event::begin());
    }

    fn handler_newround(&mut self, data: &Value) {
        let bakaze = as_usize(&data["chang"]);
        let kyoku = as_usize(&data["ju"]);
        let honba = as_usize(&data["ben"]);
        let kyoutaku = as_usize(&data["liqibang"]);
        let doras = tiles_from_mjsoul(&data["doras"]);
        let wall = as_usize(&data["left_tile_count"]);
        let mode = as_usize(&data["mode"]);

        let mut scores = [0; SEAT];
        for (s, score) in as_enumerate(&data["scores"]) {
            scores[s] = as_i32(score);
        }

        let mut hands = [vec![], vec![], vec![], vec![]];
        for s in 0..SEAT {
            if s == self.seat {
                hands[s] = tiles_from_mjsoul(&data["tiles"]);
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

        self.handle_event(Event::new(
            bakaze, kyoku, honba, kyoutaku, doras, scores, hands, wall, mode,
        ));
    }

    fn handler_dealtile(&mut self, data: &Value) {
        self.update_doras(data);
        let s = as_usize(&data["seat"]);

        if let json!(null) = &data["tile"] {
            self.handle_event(Event::deal(s, Z8));
        } else {
            let t = tile_from_mjsoul(&data["tile"]);
            self.handle_event(Event::deal(s, t));
        }
    }

    fn handler_discardtile(&mut self, data: &Value) {
        let s = as_usize(&data["seat"]);
        let t = tile_from_mjsoul(&data["tile"]);
        let m = as_bool(&data["moqie"]);
        let r = as_bool(&data["is_liqi"]);
        self.handle_event(Event::discard(s, t, m, r));
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

        let tiles = tiles_from_mjsoul(&data["tiles"]);
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

        let mut t = tile_from_mjsoul(&data["tiles"]);
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
        let stg = self.ctrl.get_stage();
        let mut doras = vec![];
        let mut ura_doras = vec![];
        let mut ctxs = vec![];
        for win in as_array(&data["hules"]) {
            let s = as_usize(&win["seat"]);
            let hand = tiles_from_mjsoul(&win["hand"]);
            let win_tile = tile_from_mjsoul(&win["hu_tile"]);
            let is_dealer = as_bool(&win["qinjia"]);
            let is_riichi = as_bool(&win["liqi"]);
            let is_drawn = as_bool(&win["zimo"]);

            let count = as_usize(&win["count"]);
            let is_yakuman = as_bool(&win["yiman"]);
            let fu = as_usize(&win["fu"]);
            let fan = if is_yakuman { 0 } else { count };
            let score = as_i32(&win["dadian"]);
            let yakuman = if is_yakuman { count } else { 0 };
            let title = get_score_title(fu, fan, yakuman);
            let points = (
                as_i32(&win["point_rong"]),
                as_i32(&win["point_zimo_xian"]),
                win["point_zimo_qin"].as_i64().unwrap_or(0) as Point,
            );

            if doras.is_empty() {
                doras = tiles_from_mjsoul(&win["doras"]);
            }
            if ura_doras.is_empty() {
                if let Value::Array(_) = win["li_doras"] {
                    ura_doras = tiles_from_mjsoul(&win["li_doras"]);
                }
            }

            let mut yakus = vec![];
            for yaku in as_array(&win["fans"]) {
                let id = as_usize(&yaku["id"]);
                let val = as_usize(&yaku["val"]);
                let stg = self.get_stage();
                let jp_wind = ["?", "東", "南", "西", "北"];
                match id {
                    10 => {
                        // 自風
                        yakus.push(Yaku {
                            name: format!("自風 {}", jp_wind[stg.get_seat_wind(s)]),
                            fan: 1,
                        });
                    }
                    11 => {
                        // 場風
                        yakus.push(Yaku {
                            name: format!("場風 {}", jp_wind[stg.get_prevalent_wind()]),
                            fan: 1,
                        });
                    }
                    _ => {
                        if let Some(y) = YakuDefine::get_from_id(id) {
                            yakus.push(Yaku {
                                name: y.name.to_string(),
                                fan: val,
                            });
                        } else {
                            error!("yaku not found: id = {}", id);
                        }
                    }
                };
            }

            let score_ctx = ScoreContext {
                yakus,
                fu,
                fan,
                yakuman,
                score,
                points,
                title,
            };

            let win_ctx = WinContext {
                seat: s,
                hand,
                win_tile,
                melds: stg.players[s].melds.clone(),
                is_dealer,
                is_drawn,
                is_riichi,
                delta_scores: [0; SEAT], // 和了毎のスコア内訳は不明
                score_context: score_ctx,
            };

            ctxs.push(win_ctx);
        }

        let mut scores = [0; SEAT];
        for (s, score) in as_enumerate(&data["old_scores"]) {
            scores[s] = as_i32(score);
        }
        let mut d_scores = [0; SEAT];
        for (s, score) in as_enumerate(&data["delta_scores"]) {
            d_scores[s] = as_i32(score);
        }

        self.handle_event(Event::win(ctxs, doras, ura_doras, scores, d_scores));
    }

    fn handler_liuju(&mut self, data: &Value) {
        let mut type_ = DrawType::Unknown;
        let mut hands = [vec![], vec![], vec![], vec![]];
        let d_scores = [0; 4];
        let nm_scores = [0; 4];
        match as_usize(&data["type"]) {
            1 => {
                // 九種九牌
                type_ = DrawType::Kyushukyuhai;
                let s = as_usize(&data["seat"]);
                hands[s] = tiles_from_mjsoul(&data["tiles"]);
            }
            2 => {
                // 四風連打
                type_ = DrawType::Suufuurenda;
            }
            3 => {
                // 四槓散了
                type_ = DrawType::Suukansanra;
            }
            4 => {
                // 四家立直
                type_ = DrawType::Suuchariichi;
            }
            5 => {
                // 三家和
                type_ = DrawType::Sanchaho;
            }
            _ => {}
        }

        self.handle_event(Event::draw(type_, hands, d_scores, nm_scores));
    }

    fn handler_notile(&mut self, data: &Value) {
        let mut d_scores = [0; SEAT];
        let nm_scores = [0; 4]; // TODO
        if let Some(ds) = &data["scores"][0]["delta_scores"].as_array() {
            for (s, score) in ds.iter().enumerate() {
                d_scores[s] = as_i32(score);
            }
        }

        let mut tenpais = [false; SEAT];
        let mut hands = [vec![], vec![], vec![], vec![]];
        for (s, player) in as_enumerate(&data["players"]) {
            tenpais[s] = as_bool(&player["tingpai"]);
            if tenpais[s] {
                hands[s] = tiles_from_mjsoul(&player["hand"]);
            }
        }

        self.handle_event(Event::draw(
            DrawType::Kouhaiheikyoku,
            hands,
            d_scores,
            nm_scores,
        ));
    }
}

fn tile_from_mjsoul2(s: &str) -> Tile {
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

fn tile_from_mjsoul(v: &Value) -> Tile {
    tile_from_mjsoul2(as_str(v))
}

fn tiles_from_mjsoul(v: &Value) -> Vec<Tile> {
    as_vec(tile_from_mjsoul, v)
}

fn calc_dapai_index(stg: &Stage, seat: Seat, tile: Tile, is_drawn: bool) -> usize {
    let pl = &stg.players[seat];
    let h = &pl.hand;
    let t = tile;
    let d = if let Some(d) = pl.drawn { d } else { Z8 };
    let is_drawn = if pl.drawn == Some(t) {
        if pl.hand[t.0][t.1] == 1 || (t.1 == 5 && pl.hand[t.0][5] == 2 && pl.hand[t.0][0] == 1) {
            // is_drawn = falseでも指定した牌がツモ牌しかない場合trueにする
            true
        } else {
            is_drawn
        }
    } else {
        // TODO: ここのコード単にfalseで良いかも
        // ツモった赤5を通常5で指定する場合に通常5がなければ赤5をツモ切り
        t.1 == 5 && pl.hand[t.0][t.1] == 1 && Some(Tile(t.0, 0)) == pl.drawn
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
fn parse_possible_action(v: &Value, stg: &Stage) -> (Vec<Action>, Vec<Index>) {
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
                    parse_combination(combs)
                } else {
                    vec![vec![]]
                };
                push(Action::new(Discard, combs[0].clone()), 0);
            }
            2 => {
                // チー
                for (idx, comb) in parse_combination(combs).iter().enumerate() {
                    push(Action::chi(comb.clone()), idx);
                }
            }
            3 => {
                // ポン
                for (idx, comb) in parse_combination(combs).iter().enumerate() {
                    push(Action::pon(comb.clone()), idx);
                }
            }
            4 => {
                // 暗槓
                for (idx, comb) in parse_combination(combs).iter().enumerate() {
                    push(Action::ankan(comb.clone()), idx);
                }
            }
            5 => {
                // 明槓
                for (idx, comb) in parse_combination(combs).iter().enumerate() {
                    push(Action::minkan(comb.clone()), idx);
                }
            }
            6 => {
                // 加槓
                // 赤5を含む場合,ponした牌の組み合わせに関係なく combs = ["0p|5p|5p|5p"] となる
                for (idx, comb) in parse_combination(combs).iter().enumerate() {
                    let mut t = comb[3];
                    if t.is_suit() && t.1 == 5 && stg.players[stg.turn].hand[t.0][0] > 0 {
                        t = Tile(t.0, 0); // 手牌に赤5があれば通常5を赤5に変換
                    }
                    push(Action::kakan(t), idx);
                }
            }
            7 => {
                // リーチ
                push(
                    Action::new(
                        ActionType::Riichi,
                        parse_combination(combs).iter().map(|x| x[0]).collect(),
                    ),
                    0,
                );
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

fn parse_combination(combs: &Value) -> Vec<Vec<Tile>> {
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
                .map(tile_from_mjsoul2)
                .collect();
            c.sort();
            c
        })
        .collect()
}
