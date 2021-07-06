use std::io;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use serde_json::Value;

use super::*;
use crate::util::common::{flush, sleep_ms, vec_remove};
use crate::util::mjai_json::*;

pub struct MjaiEndpointBuilder;

impl OperatorBuilder for MjaiEndpointBuilder {
    fn get_default_config(&self) -> Config {
        Config {
            name: "MjaiEndpoint".to_string(),
            args: vec![Arg::string("addr", "127.0.0.1:11601")],
        }
    }

    fn create(&self, config: Config) -> Box<dyn Operator> {
        Box::new(MjaiEndpoint::from_config(config))
    }
}

#[derive(Clone)]
pub struct MjaiEndpoint {
    config: Config,
    seat: usize,
    data: Arc<Mutex<SharedData>>,
    try_riichi: Option<Seat>,
    is_new_game: bool,
}

impl MjaiEndpoint {
    pub fn from_config(config: Config) -> Self {
        let data = Arc::new(Mutex::new(SharedData::new()));
        let obj = Self {
            config: config,
            seat: NO_SEAT,
            data: data.clone(),
            try_riichi: None,
            is_new_game: false,
        };

        let addr = obj.config.args[0].value.as_string();
        let listener = TcpListener::bind(addr).unwrap();
        println!("MjaiEndpoint: Listening on {}", addr);

        thread::spawn(move || {
            let is_connected = Arc::new(Mutex::new(false));
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        if *is_connected.lock().unwrap() {
                            println!("[Error] MjaiEndpoint: Duplicated connection");
                            continue;
                        }
                        *is_connected.lock().unwrap() = true;

                        let is_connected = is_connected.clone();
                        let data = data.clone();
                        thread::spawn(move || {
                            println!("MjaiEndpoint: New connection {:?}", stream);
                            match stream_handler(&mut stream, data, true) {
                                Ok(_) => {}
                                Err(e) => {
                                    println!("[Error]: {:?}", e);
                                }
                            }
                            println!("MjaiEndpoint: Connection closed");
                            *is_connected.lock().unwrap() = false;
                        });
                    }
                    Err(e) => {
                        println!("[Error] {}", e);
                    }
                }
            }
        });

        obj
    }

    fn add_record(&mut self, record: Value) {
        let mut d = self.data.lock().unwrap();
        d.selected_action = None;
        d.possible_actions = None;
        d.record.push(record);
    }

    fn confirm_riichi_accepted(&mut self, stg: &Stage) {
        if let Some(s) = self.try_riichi {
            self.try_riichi = None;
            self.add_record(mjai_reach_accepted(s, stg.get_scores()));
        }
    }

    fn notify_game_start(&mut self, _stg: &Stage, _act: &ActionGameStart) {
        assert!(self.seat != NO_SEAT);
        self.is_new_game = true;
    }

    fn notify_round_new(&mut self, stg: &Stage, act: &ActionRoundNew) {
        assert!(self.seat != NO_SEAT);
        let mut data = SharedData::new();
        if self.is_new_game {
            data.send_start_game = true;
            self.is_new_game = false;
        }
        data.seat = self.seat;
        *self.data.lock().unwrap() = data;
        self.try_riichi = None;

        // 親番の14枚目の牌は最初のツモとして扱うので取り除く
        let mut ph = act.hands.clone();
        let d = stg.players[stg.turn].drawn.unwrap();
        vec_remove(&mut ph[stg.turn], &d);

        self.add_record(mjai_start_kyoku(
            self.seat,
            act.round,
            act.kyoku,
            act.honba,
            act.kyoutaku,
            &act.doras,
            &ph,
        ));

        let act2 = ActionDealTile {
            seat: act.kyoku,
            tile: stg.players[act.kyoku].drawn.unwrap(),
        };
        self.notify_deal_tile(&stg, &act2);
    }

    fn notify_deal_tile(&mut self, stg: &Stage, act: &ActionDealTile) {
        self.confirm_riichi_accepted(stg);
        self.add_record(mjai_tsumo(self.seat, act.seat, act.tile));
    }

    fn notify_discard_tile(&mut self, _stg: &Stage, act: &ActionDiscardTile) {
        if act.is_riichi {
            self.add_record(mjai_reach(act.seat));
        }

        self.add_record(mjai_dahai(act.seat, act.tile, act.is_drawn));

        if act.is_riichi {
            self.try_riichi = Some(act.seat);
        }
    }

    fn notify_meld(&mut self, stg: &Stage, act: &ActionMeld) {
        self.confirm_riichi_accepted(stg);

        self.add_record(match act.meld_type {
            MeldType::Chi | MeldType::Pon | MeldType::Minkan => {
                let lt = stg.last_tile.unwrap();
                mjai_chiponkan(act.seat, act.meld_type, &act.consumed, lt.2, lt.0)
            }
            MeldType::Ankan => mjai_ankan(act.seat, &act.consumed),
            MeldType::Kakan => {
                let c = act.consumed[0];
                let t = c.to_normal();
                let t0 = if t.is_suit() && t.1 == 5 && c.1 != 0 {
                    Tile(t.0, 0)
                } else {
                    t
                };
                mjai_kakan(act.seat, &act.consumed, &vec![t, t, t0])
            }
        });
    }

    fn notify_kita(&mut self, _stg: &Stage, _act: &ActionKita) {
        panic!();
    }

    fn notify_dora(&mut self, _stg: &Stage, act: &ActionDora) {
        self.add_record(mjai_dora(act.tile));
    }

    fn notify_round_end_win(&mut self, stg: &Stage, act: &ActionRoundEndWin) {
        for (seat, deltas, ctx) in &act.contexts {
            self.add_record(mjai_hora(
                *seat,
                stg.turn,
                stg.last_tile.unwrap().2,
                &act.ura_doras,
                ctx,
                deltas,
                &stg.get_scores(),
            ));
        }
    }

    fn notify_round_end_draw(&mut self, stg: &Stage, act: &ActionRoundEndDraw) {
        self.add_record(mjai_ryukyoku(
            act.draw_type,
            &[false; SEAT],
            &[0; SEAT],
            &stg.get_scores(),
        ));
    }

    fn notify_round_end_no_tile(&mut self, stg: &Stage, act: &ActionRoundEndNoTile) {
        self.add_record(mjai_ryukyoku(
            DrawType::Kouhaiheikyoku,
            &act.tenpais,
            &act.points,
            &stg.get_scores(),
        ))
    }

    fn notify_game_over(&mut self, stg: &Stage, _act: &ActionGameOver) {
        self.add_record(mjai_end_game(&stg.get_scores()));
    }
}

impl Operator for MjaiEndpoint {
    fn set_seat(&mut self, seat: Seat) {
        self.seat = seat;
    }

    fn handle_operation(
        &mut self,
        stage: &Stage,
        seat: Seat,
        ops: &Vec<PlayerOperation>,
    ) -> PlayerOperation {
        // possible_actionを追加
        {
            let mut d = self.data.lock().unwrap();
            let mut acts = vec![];
            for op in ops {
                if let Some(v) = MjaiAction::from_operation(stage, seat, op) {
                    acts.push(v);
                }
            }
            d.possible_actions = Some(acts);
            d.selected_action = None;
            d.is_riichi = false;
        }

        // possible_actionに対する応答を待機
        let mut c = 0;
        loop {
            sleep_ms(100);
            let mut d = self.data.lock().unwrap();
            if d.selected_action.is_some() {
                break;
            }
            c += 1;
            if c == 50 {
                println!("[Error] possible_action timeout");
                d.possible_actions = None;
                return Op::nop();
            }
        }

        let d = &mut self.data.lock().unwrap();
        let a = std::mem::replace(&mut d.selected_action, None).unwrap();
        d.selected_action = None;

        if d.is_riichi {
            d.is_riichi = false;
            if let MjaiAction::Dahai { pai, .. } = a {
                return Op::riichi(from_mjai_tile(&pai));
            } else {
                panic!();
            }
        }

        a.to_operation(seat == stage.turn)
    }

    fn get_config(&self) -> &Config {
        &self.config
    }
}

impl StageListener for MjaiEndpoint {
    fn notify_action(&mut self, stg: &Stage, act: &Action) {
        match act {
            Action::GameStart(a) => self.notify_game_start(stg, a),
            Action::RoundNew(a) => self.notify_round_new(stg, a),
            Action::DealTile(a) => self.notify_deal_tile(stg, a),
            Action::DiscardTile(a) => self.notify_discard_tile(stg, a),
            Action::Meld(a) => self.notify_meld(stg, a),
            Action::Kita(a) => self.notify_kita(stg, a),
            Action::Dora(a) => self.notify_dora(stg, a),
            Action::RoundEndWin(a) => self.notify_round_end_win(stg, a),
            Action::RoundEndDraw(a) => self.notify_round_end_draw(stg, a),
            Action::RoundEndNoTile(a) => self.notify_round_end_no_tile(stg, a),
            Action::GameOver(a) => self.notify_game_over(stg, a),
        }
    }
}

#[derive(Debug)]
struct SharedData {
    send_start_game: bool,
    seat: Seat,
    record: Vec<Value>,
    selected_action: Option<MjaiAction>,
    possible_actions: Option<Vec<MjaiAction>>,
    is_riichi: bool,
}

impl SharedData {
    fn new() -> Self {
        Self {
            send_start_game: false,
            seat: NO_SEAT,
            record: vec![],
            selected_action: None,
            possible_actions: None,
            is_riichi: false,
        }
    }
}

fn stream_handler(
    stream: &mut TcpStream,
    data: Arc<Mutex<SharedData>>,
    debug: bool,
) -> io::Result<()> {
    let stream2 = &mut stream.try_clone().unwrap();
    let mut send = |m: &Value| send_json(stream, m, debug);
    let mut recv = || recv_json(stream2, debug);
    let err = || io::Error::new(io::ErrorKind::InvalidData, "json field not found");

    // hello
    send(&mjai_hello())?;
    let v = recv()?;

    if v["type"].as_str().ok_or_else(err)? == "join" {
        println!(
            "Player joined. name: {}",
            v["name"].as_str().ok_or_else(err)?
        );
    } else {
        println!("[Error] First message type must be 'join'");
        return Ok(());
    }

    while data.lock().unwrap().seat == NO_SEAT {
        sleep_ms(100);
    }

    let mut cursor = 0;
    let mut need_start_game = true;
    loop {
        // 初期化処理
        {
            let mut d = data.lock().unwrap();
            if cursor > d.record.len() {
                // 新しく試合開始した場合はリセット
                println!("mjai reset");
                cursor = 0;
            }
            let send_start_game = d.send_start_game;
            if cursor == 0 && (send_start_game || need_start_game) {
                // start_game 新しい試合が始まった場合,またはクライアントの再接続時に送信
                need_start_game = false;
                d.send_start_game = false;
                send(&mjai_start_game(d.seat))?;
                recv()?; // recv none
            }
        }

        let len = data.lock().unwrap().record.len();
        let mut wait_act = false;
        if cursor + 1 < len {
            send(&data.lock().unwrap().record[cursor])?;
        } else if cursor + 1 == len {
            if data.lock().unwrap().possible_actions.is_none() {
                // handle_operationがpossible_actionsを追加する可能性があるので待機
                // data.lock()が開放されている必要があることに注意
                sleep_ms(100);
            }

            let mut d = data.lock().unwrap();
            if d.possible_actions.is_some() && cursor + 1 == d.record.len() {
                // possible_actionsが存在する場合,送信用のjsonオブジェクトを生成して追加
                let a = std::mem::replace(&mut d.possible_actions, None).unwrap();
                let mut record = d.record[cursor].clone();
                record["possible_actions"] = serde_json::to_value(a).unwrap();
                d.possible_actions = None;
                wait_act = true;
                send(&record)?;
            } else {
                send(&d.record[cursor])?;
            }
        } else {
            sleep_ms(10);
            continue;
        }

        cursor += 1;

        let v = recv()?;
        if !wait_act {
            continue;
        }

        // possible_actionsに対する応答を処理
        if v["type"].as_str().ok_or_else(err)? != "reach" {
            let a = serde_json::from_value(v)?;
            data.lock().unwrap().selected_action = Some(a);
        } else {
            // reachは仕様が特殊なので個別に処理
            send(&v)?; // send reach
            let v2 = recv()?; // recv dahai
            send(&v2)?; // send dahai
            recv()?; // recv none
            {
                let d = &mut data.lock().unwrap();
                let a = serde_json::from_value(v2)?;
                d.selected_action = Some(a);
                d.is_riichi = true;
            }

            // recordに reach -> dahai -> (reach_accepted or hora) の順で追加される
            let mut step = 0;
            loop {
                sleep_ms(10);
                if data.lock().unwrap().record.len() == cursor {
                    continue;
                }

                let d = data.lock().unwrap();
                if d.record.len() < cursor {
                    // recordが別スレッドから初期化される可能性がある
                    continue;
                }
                let rec = &d.record[cursor];
                cursor += 1;
                match rec["type"].as_str().ok_or_else(err)? {
                    "reach" => {
                        assert!(step == 0);
                        step = 1;
                    }
                    "dahai" => {
                        if step == 0 {
                            // 手動でリーチ判断を無視した場合
                            break;
                        }
                        step = 2;
                    }
                    "reach_accepted" => {
                        assert!(step == 2);
                        send(rec)?; // send reach_accepted
                        recv()?; // recv none
                        break;
                    }
                    "hora" => {
                        assert!(step == 2);
                        cursor -= 1; // horaはここでは処理しない
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}

fn send_json(stream: &mut TcpStream, value: &Value, debug: bool) -> io::Result<()> {
    stream.write((value.to_string() + "\n").as_bytes())?;
    if debug {
        println!("-> {}", value.to_string());
        flush();
    }
    Ok(())
}

fn recv_json(stream: &mut TcpStream, debug: bool) -> io::Result<Value> {
    let mut buf_read = io::BufReader::new(stream);
    let mut buf = String::new();
    buf_read.read_line(&mut buf)?;
    if debug {
        println!("<- {}", buf);
        flush();
    }

    if buf.len() == 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, ""));
    }
    serde_json::from_str(&buf[..buf.len() - 1]).or_else(|e| Err(e.into()))
}
