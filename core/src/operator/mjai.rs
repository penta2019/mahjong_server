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
            "names":["Player0","Player1","Player2","Player3"]
        }));
    }

    fn notify_op_roundnew(
        &mut self,
        _stage: &Stage,
        round: usize,
        hand: usize,
        honba: usize,
        riichi_sticks: usize,
        _doras: &Vec<Tile>,
        _scores: &[i32; SEAT],
        player_hands: &[Vec<Tile>; SEAT],
    ) {
        let kaze = ["E", "S", "W", "N"];
        // let hands = vec![];
        // for hands in player_hands {
        //     for t in hands {}
        // }
        self.data.lock().unwrap().record.push(json!({
            "type":"start_kyoku",
            "bakaze": kaze[round],
            "kyoku": hand,
            "honba": honba,
            "kyoutaku": riichi_sticks,
            "dora_marker": "",
            "tehais": ""
        }));
    }

    fn notify_op_dealtile(&mut self, _stage: &Stage, _seat: Seat, _tile: Option<Tile>) {}

    fn notify_op_discardtile(
        &mut self,
        _stage: &Stage,
        _seat: Seat,
        _tile: Tile,
        _is_drawn: bool,
        _is_riichi: bool,
    ) {
    }

    fn notify_op_chiiponkan(
        &mut self,
        _stage: &Stage,
        _seat: Seat,
        _meld_type: MeldType,
        _tiles: &Vec<Tile>,
        _froms: &Vec<Seat>,
    ) {
    }

    fn notify_op_ankankakan(
        &mut self,
        _stage: &Stage,
        _seat: Seat,
        _meld_type: MeldType,
        _tile: Tile,
    ) {
    }

    fn notify_op_dora(&mut self, _stage: &Stage, _tile: Tile) {}

    fn notify_op_kita(&mut self, _stage: &Stage, _seat: Seat, _is_drawn: bool) {}

    fn notify_op_roundend_win(
        &mut self,
        _stage: &Stage,
        _ura_doras: &Vec<Tile>,
        _contexts: &Vec<(Seat, WinContext)>,
        _delta_scores: &[i32; SEAT],
    ) {
    }

    fn notify_op_roundend_draw(&mut self, _stage: &Stage, _draw_type: DrawType) {}

    fn notify_op_roundend_notile(
        &mut self,
        _stage: &Stage,
        _is_ready: &[bool; SEAT],
        _delta_scores: &[i32; SEAT],
    ) {
    }

    fn notify_op_game_over(&mut self, _stage: &Stage) {}
}

#[test]
fn test_mjai() {
    let mjai = MjaiEndpoint::new("localhost:12345");
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).ok();
}
