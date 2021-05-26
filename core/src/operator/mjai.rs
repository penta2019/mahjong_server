use std::net::{TcpListener, TcpStream};
// use std::sync::mpsc;
use std::io::prelude::*;
use std::io::{stdout, Write};
use std::sync::{Arc, Mutex};
use std::{io, thread, time};

use serde_json::{json, Value};

use crate::hand::evaluate::WinContext;
use crate::model::*;
use crate::util::operator::*;
use crate::util::stage_listener::StageListener;

use PlayerOperation::*;

const NO_SEAT: usize = 4;

// MjaiEndpoint ===============================================================

fn send_json(stream: &mut TcpStream, value: &Value) -> io::Result<()> {
    stream.write((value.to_string() + "\n").as_bytes())?;
    println!("-> {:?}", value.to_string());
    stdout().flush().unwrap();
    Ok(())
}

fn recv_json(stream: &mut TcpStream) -> io::Result<Value> {
    let mut buf_read = io::BufReader::new(stream);
    let mut buf = String::new();
    buf_read.read_line(&mut buf)?;
    println!("<- {}", buf);
    stdout().flush().unwrap();
    return Ok(serde_json::from_str(&buf[..buf.len() - 1]).unwrap());
}

fn try_unwrap<T>(v: Option<T>) -> io::Result<T> {
    match v {
        Some(v2) => Ok(v2),
        None => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "failed to unwrap Option",
        )),
    }
}

fn stream_handler(stream: &mut TcpStream, data: Arc<Mutex<SharedData>>) -> io::Result<()> {
    // hello
    let m = json!({"type":"hello","protocol":"mjsonp","protocol_version":3});
    send_json(stream, &m)?;
    let v = recv_json(stream)?;
    if try_unwrap(v["type"].as_str())? == "join" {
        println!("Player joined. name: {}", try_unwrap(v["name"].as_str())?);
    } else {
        println!("[Error] First message type must be 'join'");
        return Ok(());
    }

    // let timeout_ms = 10;
    // stream.set_read_timeout(Some(time::Duration::new(0, timeout_ms * 1000000)))?;
    let mut cursor = 0;
    loop {
        let len = data.lock().unwrap().record.len();
        let is_last_record = cursor == len - 1;
        if cursor < len - 1 {
            let d = data.lock().unwrap();
            send_json(stream, &d.record[cursor])?;
        } else if is_last_record {
            if data.lock().unwrap().possible_actions == json!(null) {
                // handle_operationがpossible_actionsを追加する可能性があるので待機
                // data.lock()が開放されている必要があることに注意
                println!("wait for possible action");
                thread::sleep(time::Duration::from_millis(10));
            }

            let d = data.lock().unwrap();
            if d.possible_actions == json!(null) {
                send_json(stream, &d.record[cursor])?;
            } else {
                // possible_actionsが存在する場合、送信用のjsonオブジェクトを生成して追加
                let mut record = d.record[cursor].clone();
                record["possible_actions"] = d.possible_actions.clone();
                send_json(stream, &record)?;
            }
        } else {
            thread::sleep(time::Duration::from_millis(10));
            continue;
        }
        cursor += 1;

        match recv_json(stream) {
            Ok(v) => {
                if is_last_record {
                    data.lock().unwrap().action = v;
                }
            }
            Err(e) => return Err(e),
        }
    }
}

#[derive(Debug)]
struct SharedData {
    record: Vec<Value>,
    action: Value,
    possible_actions: Value,
}

#[derive(Clone)]
pub struct MjaiEndpoint {
    seat: usize,
    data: Arc<Mutex<SharedData>>,
}

impl MjaiEndpoint {
    pub fn new(addr: &str) -> Self {
        let shared_data = SharedData {
            record: vec![],
            action: json!(null),
            possible_actions: json!(null),
        };
        let data = Arc::new(Mutex::new(shared_data));
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
                            match stream_handler(&mut stream, data) {
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

    pub fn set_seat(&mut self, seat: usize) {
        self.seat = seat;
    }

    fn add_record(&mut self, record: Value) {
        let mut d = self.data.lock().unwrap();
        d.action = json!(null);
        d.possible_actions = json!(null);
        d.record.push(record);
    }
}

impl Operator for MjaiEndpoint {
    fn handle_operation(
        &mut self,
        _stage: &Stage,
        _seat: Seat,
        _ops: &Vec<PlayerOperation>,
    ) -> PlayerOperation {
        {
            let mut d = self.data.lock().unwrap();
            d.possible_actions = json!([]);
        }

        // let mut buf = String::new();
        // std::io::stdin().read_line(&mut buf).ok();

        loop {
            thread::sleep(time::Duration::from_millis(20));
            if self.data.lock().unwrap().action != json!(null) {
                break;
            }
        }
        println!("client action: {:?}", self.data.lock().unwrap().action);
        Nop
    }

    fn debug_string(&self) -> String {
        "MjaiEndpoint".to_string()
    }
}

impl StageListener for MjaiEndpoint {
    fn notify_op_game_start(&mut self, _stage: &Stage) {
        assert!(self.seat != NO_SEAT);
        self.add_record(json!({
            "type":"start_game",
            "id":self.seat,
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
        let dora_marker = mjai_tile_symbol(doras[0]);

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
            mjai_tile_symbol(tile.unwrap())
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
            "pai": mjai_tile_symbol(tile),
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
                pai = mjai_tile_symbol(t);
            } else {
                target = f;
                consumed.push(mjai_tile_symbol(t));
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
                    consumed.push(mjai_tile_symbol(t))
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
                        pai = mjai_tile_symbol(t);
                    } else {
                        consumed.push(mjai_tile_symbol(t))
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
            "dora_marker": mjai_tile_symbol(tile),
        }));
    }

    fn notify_op_kita(&mut self, _stage: &Stage, _seat: Seat, _is_drawn: bool) {
        panic!();
    }

    fn notify_op_roundend_win(
        &mut self,
        stage: &Stage,
        ura_doras: &Vec<Tile>,
        contexts: &Vec<(Seat, WinContext)>,
        score_deltas: &[i32; SEAT],
    ) {
        let ura: Vec<String> = ura_doras.iter().map(|&t| mjai_tile_symbol(t)).collect();
        for (seat, ctx) in contexts {
            self.add_record(json!({
                "type": "hora",
                "actor": seat,
                "target": stage.turn,
                "pai": mjai_tile_symbol(stage.last_tile.unwrap().1),
                "uradora_markers": ura,
                "hora_tehais": [], // TODO
                "yakus": [], // TODO
                "fu": ctx.fu,
                "fan": ctx.fan_mag,
                "hora_points": ctx.pay_scores.0,
                "deltas": score_deltas,
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
        score_deltas: &[i32; SEAT],
    ) {
        self.add_record(json!({
            "type": "ryukyoku",
            "reason": "fanpai",
            "tehais": [], // TODO
            "tenpais": is_ready,
            "deltas": score_deltas,
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

// Utility ====================================================================

fn mjai_tile_symbol(t: Tile) -> String {
    if t.0 == TZ {
        assert!(WE <= t.1 && t.1 <= DR);
        let hornor = ["", "E", "S", "W", "N", "P", "F", "C"];
        return hornor[t.1].to_string();
    } else {
        let tile_type = ["m", "p", "s"];
        return format!(
            "{}{}{}",
            t.n(),
            tile_type[t.0],
            if t.1 == 0 { "r" } else { "" }
        );
    }
}

fn from_mjai_tile_symbol(sym: &str) -> Tile {
    match sym {
        "?" => Z8,
        "E" => Tile(TZ, WE),
        "S" => Tile(TZ, WS),
        "W" => Tile(TZ, WW),
        "N" => Tile(TZ, WN),
        "P" => Tile(TZ, DW),
        "F" => Tile(TZ, DG),
        "C" => Tile(TZ, DR),
        _ => {
            let sym = sym.as_bytes();
            let ti = match sym[1] {
                b'm' => 0,
                b'p' => 1,
                b's' => 2,
                _ => panic!(),
            } as usize;
            let mut ni = (sym[0] - b'0') as usize;
            if ni == 5 && sym.len() == 3 {
                ni = 0;
            }
            assert!(ni < TNUM);
            Tile(ti, ni)
        }
    }
}

fn create_tehais(player_hands: &[Vec<Tile>; SEAT], seat: usize) -> Vec<Vec<String>> {
    let mut hands = vec![];
    for (seat2, hands2) in player_hands.iter().enumerate() {
        let mut hand = vec![];
        for &t in hands2 {
            if seat == seat2 {
                hand.push(mjai_tile_symbol(t));
            } else {
                hand.push("?".to_string());
            }
        }
        hands.push(hand);
    }
    hands
}

#[test]
fn test_mjai() {
    let mjai = MjaiEndpoint::new("127.0.0.1:12345");
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).ok();
}

#[test]
fn test_mjai2() {
    println!("{}", from_mjai_tile_symbol("?"));
}
