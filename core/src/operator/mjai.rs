use std::net::{TcpListener, TcpStream};
// use std::sync::mpsc;
use std::io::prelude::*;
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
    stream.write(value.to_string().as_bytes())?;
    Ok(())
}

fn recv_json(stream: &mut TcpStream) -> io::Result<Value> {
    let mut buf = [0; 4096];
    stream.read(&mut buf)?;
    if let Ok(s) = std::str::from_utf8(&buf) {
        if let Some(r) = s.rfind('}') {
            return Ok(serde_json::from_str(&s[..r + 1])?);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "Failed to convert input into json value",
    ))
}

// fn try_recv_json(stream: &mut TcpStream) -> io::Result<Value> {
//     stream.set_nonblocking(true)?;
//     let v = recv_json(stream)?;
//     stream.set_nonblocking(false)?;
//     Ok(v)
// }

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

    let timeout_ms = 10;
    stream.set_read_timeout(Some(time::Duration::new(0, timeout_ms * 1000000)))?;
    let mut cursor = 0;
    loop {
        {
            let d = data.lock().unwrap();
            if cursor < d.record.len() {
                send_json(stream, &d.record[cursor])?;
                cursor += 1;
            }
        }

        match recv_json(stream) {
            Ok(v) => match try_unwrap(v["type"].as_str())? {
                "none" => {}
                m => println!("Unknown message type: {}", m),
            },
            Err(e) => match e.kind() {
                io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => {
                    // linux: WouldBlock, windows: TimedOut
                }
                _ => return Err(e),
            },
        }
    }
    // Ok(())
}

#[derive(Default)]
struct SharedData {
    record: Vec<Value>,
}

#[derive(Clone)]
pub struct MjaiEndpoint {
    seat: usize,
    data: Arc<Mutex<SharedData>>,
}

impl MjaiEndpoint {
    pub fn new(addr: &str) -> Self {
        let data = Arc::new(Mutex::new(SharedData::default()));
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
}

impl Operator for MjaiEndpoint {
    fn handle_operation(
        &mut self,
        _stage: &Stage,
        _seat: Seat,
        _ops: &Vec<PlayerOperation>,
    ) -> PlayerOperation {
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).ok();
        Nop
    }

    fn debug_string(&self) -> String {
        "MjaiEndpoint".to_string()
    }
}

impl StageListener for MjaiEndpoint {
    fn notify_op_game_start(&mut self, _stage: &Stage) {
        assert!(self.seat != NO_SEAT);
        self.data.lock().unwrap().record.push(json!({
            "type":"start_game",
            "id":self.seat,
            "names":["Player0","Player1","Player2","Player3"],
        }));
    }

    fn notify_op_roundnew(
        &mut self,
        _stage: &Stage,
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

        // let dora
        self.data.lock().unwrap().record.push(json!({
            "type":"start_kyoku",
            "bakaze": wind[round],
            "kyoku": kyoku,
            "honba": honba,
            "kyotaku": kyoutaku,
            "dora_marker": dora_marker,
            "tehais": hands,
        }));
    }

    fn notify_op_dealtile(&mut self, _stage: &Stage, seat: Seat, tile: Option<Tile>) {
        let t = if self.seat == seat {
            mjai_tile_symbol(tile.unwrap())
        } else {
            "?".to_string()
        };
        self.data.lock().unwrap().record.push(json!({
            "type": "tsumo",
            "actor": seat,
            "pai": t,
        }));
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
            self.data.lock().unwrap().record.push(json!({
                "type": "reach",
                "actor": seat,
            }));
        }

        self.data.lock().unwrap().record.push(json!({
            "type": "dahai",
            "actor": seat,
            "pai": mjai_tile_symbol(tile),
            "tsumogiri": is_drawn,
        }));
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
        self.data.lock().unwrap().record.push(json!({
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
                self.data.lock().unwrap().record.push(json!({
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

                self.data.lock().unwrap().record.push(json!({
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
        self.data.lock().unwrap().record.push(json!({
            "type": "dora",
            "dora_marker": mjai_tile_symbol(tile),
        }));
    }

    fn notify_op_kita(&mut self, _stage: &Stage, _seat: Seat, _is_drawn: bool) {
        panic!();
    }

    fn notify_op_roundend_win(
        &mut self,
        _stage: &Stage,
        _ura_doras: &Vec<Tile>,
        contexts: &Vec<(Seat, WinContext)>,
        score_deltas: &[i32; SEAT],
    ) {
        for (seat, ctx) in contexts {
            // TODO
            self.data.lock().unwrap().record.push(json!({
                "type": "hora",
                "actor": seat,
            }));
        }
    }

    fn notify_op_roundend_draw(&mut self, _stage: &Stage, _draw_type: DrawType) {}

    fn notify_op_roundend_notile(
        &mut self,
        _stage: &Stage,
        _is_ready: &[bool; SEAT],
        _score_deltas: &[i32; SEAT],
    ) {
    }

    fn notify_op_game_over(&mut self, _stage: &Stage) {}
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
    let mjai = MjaiEndpoint::new("localhost:12345");
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).ok();
}
