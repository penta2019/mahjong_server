use std::io;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use serde_json::{json, Value};

use super::*;
use crate::hand::evaluate::WinContext;
use crate::util::common::sleep_ms;
use crate::util::mjai_json::*;

const NO_SEAT: usize = 4;

// MjaiEndpoint ===============================================================
#[derive(Clone)]
pub struct MjaiEndpoint {
    seat: usize,
    data: Arc<Mutex<SharedData>>,
}

impl MjaiEndpoint {
    pub fn new(addr: &str) -> Self {
        let data = Arc::new(Mutex::new(SharedData::new()));
        let obj = Self {
            seat: NO_SEAT,
            data: data.clone(),
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
        d.action = json!(null);
        d.possible_actions = json!(null);
        d.record.push(record);
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
                if let Some(v) = ClientMessage::from_operation_to_value(stage, seat, op) {
                    acts.push(v);
                }
            }
            d.possible_actions = json!(acts);
        }

        // possible_actionに対する応答を待機
        let mut c = 0;
        loop {
            sleep_ms(100);
            if self.data.lock().unwrap().action != json!(null) {
                break;
            }
            c += 1;
            if c == 50 {
                println!("[Error] possible_action timeout");
                return Op::nop();
            }
        }

        let v = std::mem::replace(&mut self.data.lock().unwrap().action, json!(null));

        // reachだけはmjaiと仕様が異なるため個別処理する
        if v["type"] == json!("reach") {
            if let Some(t) = v["pai"].as_str() {
                return Op::riichi(from_mjai_tile(t));
            } else {
                println!("[Error] pai not found in reach message");
                return Op::nop();
            }
        }

        if let Ok(cmsg) = ClientMessage::from_value_to_operation(v, seat == stage.turn) {
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
        *self.data.lock().unwrap() = SharedData::new();
        self.add_record(json!({
            "type":"start_game",
            "id": self.seat,
            "names":["Player0","Player1","Player2","Player3"],
        }));
    }

    fn notify_op_roundnew(
        &mut self,
        stage: &Stage,
        round: usize,
        kyoku: usize,
        honba: usize,
        kyoutaku: usize,
        doras: &Vec<Tile>,
        _scores: &[i32; SEAT],
        player_hands: &[Vec<Tile>; SEAT],
    ) {
        let wind = ["E", "S", "W", "N"];
        let hands = create_tehais(player_hands, self.seat);

        assert!(doras.len() == 1);
        let dora_marker = to_mjai_tile(doras[0]);

        self.add_record(json!({
            "type":"start_kyoku",
            "bakaze": wind[round],
            "kyoku": kyoku,
            "honba": honba,
            "kyotaku": kyoutaku,
            "dora_marker": dora_marker,
            "tehais": hands,
        }));

        self.notify_op_dealtile(stage, kyoku, stage.players[kyoku].drawn);
    }

    fn notify_op_dealtile(&mut self, _stage: &Stage, seat: Seat, tile: Option<Tile>) {
        let t = if self.seat == seat {
            to_mjai_tile(tile.unwrap())
        } else {
            "?".to_string()
        };
        self.add_record(json!({
            "type": "tsumo",
            "actor": seat,
            "pai": t,
        }));
    }

    fn notify_op_discardtile(
        &mut self,
        stage: &Stage,
        seat: Seat,
        tile: Tile,
        is_drawn: bool,
        is_riichi: bool,
    ) {
        if is_riichi {
            self.add_record(json!({
                "type": "reach",
                "actor": seat,
            }));
        }

        self.add_record(json!({
            "type": "dahai",
            "actor": seat,
            "pai": to_mjai_tile(tile),
            "tsumogiri": is_drawn,
        }));

        if is_riichi {
            let mut deltas = [0, 0, 0, 0];
            deltas[seat] = -1000;
            self.add_record(json!({
                "type": "reach_accepted",
                "actor": seat,
                "deltas": deltas,
                "scores": stage.get_scores(),
            }));
        }
    }

    fn notify_op_chiiponkan(
        &mut self,
        _stage: &Stage,
        seat: Seat,
        meld_type: MeldType,
        tiles: &Vec<Tile>,
        froms: &Vec<Seat>,
    ) {
        let mut consumed = vec![];
        let mut pai = "".to_string();
        let mut target = NO_SEAT;
        for (&t, &f) in tiles.iter().zip(froms.iter()) {
            if seat == f {
                consumed.push(to_mjai_tile(t));
            } else {
                target = f;
                pai = to_mjai_tile(t);
            }
        }
        assert!(pai != "" && target != NO_SEAT);

        let type_ = match meld_type {
            MeldType::Chii => "chi",
            MeldType::Pon => "pon",
            MeldType::Minkan => "daiminkan",
            _ => panic!(),
        };
        self.add_record(json!({
            "type": type_,
            "actor": seat,
            "pai": pai,
            "target": target,
            "consumed": consumed,
        }));
    }

    fn notify_op_ankankakan(&mut self, stage: &Stage, seat: Seat, meld_type: MeldType, tile: Tile) {
        let pl = &stage.players[seat];
        let meld = pl.melds.iter().find(|m| m.tiles.contains(&tile)).unwrap();
        match meld_type {
            MeldType::Ankan => {
                let mut consumed = vec![];
                for &t in meld.tiles.iter() {
                    consumed.push(to_mjai_tile(t))
                }
                self.add_record(json!({
                    "type": "ankan",
                    "actor": seat,
                    "consumed": consumed,
                }));
            }
            MeldType::Kakan => {
                let mut pai = "".to_string();
                let mut consumed = vec![];
                for &t in meld.tiles.iter() {
                    if pai == "" && t == tile {
                        pai = to_mjai_tile(t);
                    } else {
                        consumed.push(to_mjai_tile(t))
                    }
                }
                assert!(pai != "");

                self.add_record(json!({
                    "type": "kakan",
                    "actor": seat,
                    "pai": pai,
                    "consumed": consumed,
                }));
            }
            _ => panic!(),
        }
    }

    fn notify_op_dora(&mut self, _stage: &Stage, tile: Tile) {
        self.add_record(json!({
            "type": "dora",
            "dora_marker": to_mjai_tile(tile),
        }));
    }

    fn notify_op_kita(&mut self, _stage: &Stage, _seat: Seat, _is_drawn: bool) {
        panic!();
    }

    fn notify_op_roundend_win(
        &mut self,
        stage: &Stage,
        ura_doras: &Vec<Tile>,
        contexts: &Vec<(Seat, [i32; SEAT], WinContext)>,
    ) {
        let ura: Vec<String> = ura_doras.iter().map(|&t| to_mjai_tile(t)).collect();
        for (seat, deltas, ctx) in contexts {
            self.add_record(json!({
                "type": "hora",
                "actor": seat,
                "target": stage.turn,
                "pai": to_mjai_tile(stage.last_tile.unwrap().2),
                "uradora_markers": ura,
                "hora_tehais": [], // TODO
                "yakus": [], // TODO
                "fu": ctx.fu,
                "fan": ctx.fan_mag,
                "hora_points": ctx.pay_scores.0,
                "deltas": deltas,
                "scores": stage.get_scores(),
            }));
        }
    }

    fn notify_op_roundend_draw(&mut self, stage: &Stage, _draw_type: DrawType) {
        self.add_record(json!({
            "type": "ryukyoku",
            "reason": "", // TODO
            "tehais": [], // TODO
            "tenpais": [false, false, false, false],
            "deltas": [0, 0, 0, 0],
            "scores": stage.get_scores(),
        }));
    }

    fn notify_op_roundend_notile(
        &mut self,
        stage: &Stage,
        is_ready: &[bool; SEAT],
        delta_scores: &[i32; SEAT],
    ) {
        self.add_record(json!({
            "type": "ryukyoku",
            "reason": "fanpai",
            "tehais": [], // TODO
            "tenpais": is_ready,
            "deltas": delta_scores,
            "scores": stage.get_scores(),
        }));
    }

    fn notify_op_game_over(&mut self, stage: &Stage) {
        self.add_record(json!({
            "type": "end_game",
            "scores": stage.get_scores(),
        }));
    }
}

#[derive(Debug)]
struct SharedData {
    record: Vec<Value>,
    action: Value,
    possible_actions: Value,
}

impl SharedData {
    fn new() -> Self {
        Self {
            record: vec![],
            action: json!(null),
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
    let m = json!({"type":"hello","protocol":"mjsonp","protocol_version":3});
    send(&m)?;
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

    let mut cursor = 0;
    let mut reach_skip = false;
    loop {
        if cursor > data.lock().unwrap().record.len() {
            // 新しく試合開始した場合はリセット
            println!("mjai reset");
            cursor = 0;
        }

        while reach_skip {
            if cursor == data.lock().unwrap().record.len() {
                sleep_ms(10);
                continue;
            }

            let rec = &data.lock().unwrap().record[cursor];
            cursor += 1;
            if rec["type"].as_str().ok_or_else(err)? == "reach_accepted" {
                reach_skip = false;
                break;
            }
        }

        let len = data.lock().unwrap().record.len();
        let is_last_record = cursor + 1 == len;
        // println!("cursor: {}", cursor);
        if cursor + 1 < len {
            send(&data.lock().unwrap().record[cursor])?;
        } else if is_last_record {
            if data.lock().unwrap().possible_actions == json!(null) {
                // handle_operationがpossible_actionsを追加する可能性があるので待機
                // data.lock()が開放されている必要があることに注意
                sleep_ms(1000);
            }

            let d = data.lock().unwrap();
            if d.possible_actions == json!(null) {
                send(&d.record[cursor])?;
            } else {
                // possible_actionsが存在する場合、送信用のjsonオブジェクトを生成して追加
                let mut record = d.record[cursor].clone();
                record["possible_actions"] = d.possible_actions.clone();
                send(&record)?;
            }
        } else {
            sleep_ms(10);
            continue;
        }
        cursor += 1;

        let mut d = data.lock().unwrap();
        let v = recv()?;
        if is_last_record && d.possible_actions != json!(null) {
            if v["type"].as_str().ok_or_else(err)? == "reach" {
                send(&v)?; // send reach
                let mut v2 = recv()?; // recv dahai
                send(&v2)?; // send dahai
                recv()?; // recv none

                // send reach_accepted
                let actor = v["actor"].as_u64().ok_or_else(err)? as usize;
                let mut deltas = [0; SEAT];
                deltas[actor] = -1000;
                send(&json!({
                    "type":"reach_accepted",
                    "actor": actor,
                    "deltas": deltas,
                    "scores":[25000, 25000, 25000, 25000],
                }))?;
                recv()?; // recv none
                reach_skip = true;

                v2["type"] = json!("reach");
                d.action = v2;
            } else {
                d.action = v;
            }
        }
    }
}

fn send_json(stream: &mut TcpStream, value: &Value, debug: bool) -> io::Result<()> {
    stream.write((value.to_string() + "\n").as_bytes())?;
    if debug {
        println!("-> {:?}", value.to_string());
        io::stdout().flush().unwrap();
    }
    Ok(())
}

fn recv_json(stream: &mut TcpStream, debug: bool) -> io::Result<Value> {
    let err = || -> io::Result<Value> {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "recv invalid json data",
        ))
    };
    let mut buf_read = io::BufReader::new(stream);
    let mut buf = String::new();
    buf_read.read_line(&mut buf)?;
    if debug {
        println!("<- {}", buf);
        io::stdout().flush().unwrap();
    }

    if buf.len() == 0 {
        err()?;
    }
    serde_json::from_str(&buf[..buf.len() - 1]).or_else(|_| err())
}
