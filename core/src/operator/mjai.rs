use std::io;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use serde_json::{json, Value};

use super::*;
use crate::hand::evaluate::WinContext;
use crate::model::*;
use crate::util::common::{sleep_ms, vec_remove};
use crate::util::mjai_json::*;

// MjaiEndpoint ===============================================================
#[derive(Clone)]
pub struct MjaiEndpoint {
    seat: usize,
    data: Arc<Mutex<SharedData>>,
    try_riichi: Option<Seat>,
    is_new_game: bool,
}

impl MjaiEndpoint {
    pub fn new(addr: &str) -> Self {
        let data = Arc::new(Mutex::new(SharedData::new()));
        let obj = Self {
            seat: NO_SEAT,
            data: data.clone(),
            try_riichi: None,
            is_new_game: false,
        };
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
        d.selected_action = json!(null);
        d.possible_actions = json!(null);
        d.record.push(record);
    }

    fn check_riichi_accepted(&mut self, stage: &Stage) {
        if let Some(s) = self.try_riichi {
            self.try_riichi = None;
            self.add_record(mjai_reach_accepted(s, stage.get_scores()));
        }
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
                if let Some(v) = MjaiAction::from_operation_to_value(stage, seat, op) {
                    acts.push(v);
                }
            }
            d.possible_actions = json!(acts);
            d.selected_action = json!(null);
        }

        // possible_actionに対する応答を待機
        let mut c = 0;
        loop {
            sleep_ms(100);
            let mut d = self.data.lock().unwrap();
            if d.selected_action != json!(null) {
                break;
            }
            c += 1;
            if c == 50 {
                println!("[Error] possible_action timeout");
                d.possible_actions = json!(null);
                return Op::nop();
            }
        }

        let v = std::mem::replace(&mut self.data.lock().unwrap().selected_action, json!(null));

        // reachだけはmjaiと仕様が異なるため個別処理する
        if v["type"] == json!("reach") {
            if let Some(t) = v["pai"].as_str() {
                return Op::riichi(from_mjai_tile(t));
            } else {
                println!("[Error] pai not found in reach message");
                return Op::nop();
            }
        }

        if let Ok(cmsg) = MjaiAction::from_value_to_operation(v, seat == stage.turn) {
            cmsg
        } else {
            println!("[Error] Failed to parse mjai action");
            Op::nop()
        }
    }

    fn name(&self) -> String {
        "MjaiEndpoint".to_string()
    }
}

impl StageListener for MjaiEndpoint {
    fn notify_op_game_start(&mut self, _stage: &Stage) {
        assert!(self.seat != NO_SEAT);
        self.is_new_game = true;
    }

    fn notify_op_roundnew(
        &mut self,
        stage: &Stage,
        round: usize,
        kyoku: usize,
        honba: usize,
        kyoutaku: usize,
        doras: &Vec<Tile>,
        _scores: &[Score; SEAT],
        player_hands: &[Vec<Tile>; SEAT],
    ) {
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
        let mut ph = player_hands.clone();
        let d = stage.players[stage.turn].drawn.unwrap();
        vec_remove(&mut ph[stage.turn], &d);

        self.add_record(mjai_start_kyoku(
            self.seat, round, kyoku, honba, kyoutaku, doras, &ph,
        ));

        self.notify_op_dealtile(stage, kyoku, stage.players[kyoku].drawn.unwrap());
    }

    fn notify_op_dealtile(&mut self, stage: &Stage, seat: Seat, tile: Tile) {
        self.check_riichi_accepted(stage);
        self.add_record(mjai_tsumo(self.seat, seat, tile));
    }

    fn notify_op_discardtile(
        &mut self,
        _stage: &Stage,
        seat: Seat,
        tile: Tile,
        is_drawn: bool,
        is_riichi: bool,
    ) {
        if is_riichi {
            self.add_record(mjai_reach(seat));
        }

        self.add_record(mjai_dahai(seat, tile, is_drawn));

        if is_riichi {
            self.try_riichi = Some(seat);
        }
    }

    fn notify_op_chiiponkan(
        &mut self,
        stage: &Stage,
        seat: Seat,
        meld_type: MeldType,
        tiles: &Vec<Tile>,
        froms: &Vec<Seat>,
    ) {
        self.check_riichi_accepted(stage);
        self.add_record(mjai_chiponkan(seat, meld_type, tiles, froms));
    }

    fn notify_op_ankankakan(&mut self, stage: &Stage, seat: Seat, meld_type: MeldType, tile: Tile) {
        let tiles = &stage.players[seat]
            .melds
            .iter()
            .find(|m| m.tiles.contains(&tile))
            .unwrap()
            .tiles;
        self.add_record(mjai_ankankakan(seat, meld_type, tile, tiles));
    }

    fn notify_op_dora(&mut self, _stage: &Stage, tile: Tile) {
        self.add_record(mjai_dora(tile));
    }

    fn notify_op_kita(&mut self, _stage: &Stage, _seat: Seat, _is_drawn: bool) {
        panic!();
    }

    fn notify_op_roundend_win(
        &mut self,
        stage: &Stage,
        ura_doras: &Vec<Tile>,
        contexts: &Vec<(Seat, [Score; SEAT], WinContext)>,
    ) {
        for (seat, deltas, ctx) in contexts {
            self.add_record(mjai_hora(
                *seat,
                stage.turn,
                stage.last_tile.unwrap().2,
                ura_doras,
                ctx,
                deltas,
                &stage.get_scores(),
            ));
        }
    }

    fn notify_op_roundend_draw(&mut self, stage: &Stage, draw_type: DrawType) {
        self.add_record(mjai_ryukyoku(
            draw_type,
            &[false; SEAT],
            &[0; SEAT],
            &stage.get_scores(),
        ));
    }

    fn notify_op_roundend_notile(
        &mut self,
        stage: &Stage,
        is_ready: &[bool; SEAT],
        delta_scores: &[Score; SEAT],
    ) {
        self.add_record(mjai_ryukyoku(
            DrawType::Kouhaiheikyoku,
            is_ready,
            delta_scores,
            &stage.get_scores(),
        ))
    }

    fn notify_op_game_over(&mut self, stage: &Stage) {
        self.add_record(mjai_end_game(&stage.get_scores()));
    }
}

#[derive(Debug)]
struct SharedData {
    send_start_game: bool,
    seat: Seat,
    record: Vec<Value>,
    selected_action: Value,
    possible_actions: Value,
}

impl SharedData {
    fn new() -> Self {
        Self {
            send_start_game: false,
            seat: NO_SEAT,
            record: vec![],
            selected_action: json!(null),
            possible_actions: json!(null),
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
                // start_game 新しい試合が始まった場合、またはクライアントの再接続時に送信
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
            if data.lock().unwrap().possible_actions == json!(null) {
                // handle_operationがpossible_actionsを追加する可能性があるので待機
                // data.lock()が開放されている必要があることに注意
                sleep_ms(100);
            }

            let mut d = data.lock().unwrap();
            if d.possible_actions != json!(null) && cursor + 1 == d.record.len() {
                // possible_actionsが存在する場合、送信用のjsonオブジェクトを生成して追加
                let mut record = d.record[cursor].clone();
                record["possible_actions"] = d.possible_actions.clone();
                d.possible_actions = json!(null);
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
            data.lock().unwrap().selected_action = v;
        } else {
            // reachは仕様が特殊なので個別に処理
            send(&v)?; // send reach
            let mut v2 = recv()?; // recv dahai
            send(&v2)?; // send dahai
            recv()?; // recv none
            v2["type"] = json!("reach");
            data.lock().unwrap().selected_action = v2;

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
        io::stdout().flush().unwrap();
    }
    Ok(())
}

fn recv_json(stream: &mut TcpStream, debug: bool) -> io::Result<Value> {
    let mut buf_read = io::BufReader::new(stream);
    let mut buf = String::new();
    buf_read.read_line(&mut buf)?;
    if debug {
        println!("<- {}", buf);
        io::stdout().flush().unwrap();
    }

    if buf.len() == 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, ""));
    }
    serde_json::from_str(&buf[..buf.len() - 1]).or_else(|e| Err(e.into()))
}
