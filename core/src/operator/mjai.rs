use std::io;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use serde::{Deserialize, Serialize};
use serde_json::{json, Result, Value};

use super::*;
use crate::hand::evaluate::WinContext;
use crate::util::common::sleep_ms;

use PlayerOperationType::*;

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
                let PlayerOperation(tp, cs) = op;
                match tp {
                    Nop => {}
                    Discard => {}
                    Ankan => {
                        acts.push(json!(MsgAnkan {
                            type_: "ankan".to_string(),
                            actor: seat,
                            consumed: vec_to_mjai_tile(cs),
                        }));
                    }
                    Kakan => {
                        let t = op.1[0];
                        let comsumed = if t.1 == 0 {
                            // 赤5
                            let t2 = Tile(t.0, 5);
                            vec![t2, t2, t2]
                        } else if t.is_suit() && t.1 == 5 {
                            // 通常5
                            vec![Tile(t.0, 0), t, t]
                        } else {
                            vec![t, t, t]
                        }
                        .iter()
                        .map(|&t| to_mjai_tile(t))
                        .collect();

                        acts.push(json!(MsgKakan {
                            type_: "kakan".to_string(),
                            actor: seat,
                            pai: to_mjai_tile(t),
                            consumed: comsumed,
                        }));
                    }
                    Riichi => {}
                    Tsumo => {
                        acts.push(json!(MsgHora {
                            type_: "hora".to_string(),
                            actor: seat,
                            target: seat,
                            pai: to_mjai_tile(stage.players[seat].drawn.unwrap()),
                        }));
                    }
                    Kyushukyuhai => {
                        acts.push(json!(MsgRyukyoku {
                            type_: "ryukyoku".to_string(),
                            actor: seat,
                            reason: "kyushukyuhai".to_string(),
                        }));
                    }
                    Kita => {}
                    Chii => {
                        let (target_seat, _, target_tile) = stage.last_tile.unwrap();
                        acts.push(json!(MsgChi {
                            type_: "chi".to_string(),
                            actor: seat,
                            target: target_seat,
                            pai: to_mjai_tile(target_tile),
                            consumed: vec_to_mjai_tile(cs),
                        }));
                    }
                    Pon => {
                        let (target_seat, _, target_tile) = stage.last_tile.unwrap();
                        acts.push(json!(MsgPon {
                            type_: "pon".to_string(),
                            actor: seat,
                            target: target_seat,
                            pai: to_mjai_tile(target_tile),
                            consumed: vec_to_mjai_tile(cs),
                        }));
                    }
                    Minkan => {
                        let (target_seat, _, target_tile) = stage.last_tile.unwrap();
                        acts.push(json!(MsgDaiminkan {
                            type_: "daiminkan".to_string(),
                            actor: seat,
                            target: target_seat,
                            pai: to_mjai_tile(target_tile),
                            consumed: vec_to_mjai_tile(cs),
                        }));
                    }
                    Ron => {
                        let lt = stage.last_tile.unwrap();
                        acts.push(json!(MsgHora {
                            type_: "hora".to_string(),
                            actor: seat,
                            target: lt.0,
                            pai: to_mjai_tile(lt.2),
                        }));
                    }
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

        let cmsg = ClientMessage::from_value(v);
        if let Err(_) = cmsg {
            println!("[Error] Failed to parse mjai action");
            return Op::nop();
        }

        let cmsg = cmsg.unwrap();
        let op = match cmsg.type_ {
            MsgType::Dahai => {
                let m = cmsg.dahai.unwrap();
                if m.tsumogiri {
                    Op::nop()
                } else {
                    Op::discard(from_mjai_tile(&m.pai))
                }
            }
            MsgType::Chi => {
                let m = cmsg.chi.unwrap();
                Op::chii(vec_from_mjai_tile(&m.consumed))
            }
            MsgType::Pon => {
                let m = cmsg.pon.unwrap();
                Op::pon(vec_from_mjai_tile(&m.consumed))
            }
            MsgType::Kakan => {
                let m = cmsg.kakan.unwrap();
                Op::kakan(from_mjai_tile(&m.pai))
            }
            MsgType::Daiminkan => {
                let m = cmsg.daiminkan.unwrap();
                Op::minkan(vec_from_mjai_tile(&m.consumed))
            }
            MsgType::Ankan => {
                let m = cmsg.ankan.unwrap();
                Op::ankan(vec_from_mjai_tile(&m.consumed))
            }
            MsgType::Reach => panic!(),
            MsgType::Hora => {
                if ops.contains(&Op::tsumo()) {
                    Op::tsumo()
                } else if ops.contains(&Op::ron()) {
                    Op::ron()
                } else {
                    println!("[Error] Invalid hora action.");
                    return Op::nop();
                }
            }
            MsgType::Ryukyoku => Op::kyushukyuhai(),
            MsgType::None => Op::nop(),
        };

        // println!("mjai operation: {:?}", op);

        op
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

// Utility ====================================================================

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

fn to_mjai_tile(t: Tile) -> String {
    if t.is_hornor() {
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

fn from_mjai_tile(sym: &str) -> Tile {
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

fn vec_to_mjai_tile(v: &Vec<Tile>) -> Vec<String> {
    v.iter().map(|&t| to_mjai_tile(t)).collect()
}

fn vec_from_mjai_tile(v: &Vec<String>) -> Vec<Tile> {
    v.iter().map(|t| from_mjai_tile(t)).collect()
}

fn create_tehais(player_hands: &[Vec<Tile>; SEAT], seat: usize) -> Vec<Vec<String>> {
    let mut hands = vec![];
    for (seat2, hands2) in player_hands.iter().enumerate() {
        let mut hand = vec![];
        for &t in hands2 {
            if seat == seat2 {
                hand.push(to_mjai_tile(t));
            } else {
                hand.push("?".to_string());
            }
        }
        hands.push(hand);
    }
    hands
}

// Mjai client message =======================================================

#[derive(Debug)]
enum MsgType {
    Dahai,
    Pon,
    Chi,
    Kakan,
    Daiminkan,
    Ankan,
    Reach,
    Hora,
    Ryukyoku,
    None,
}

impl Default for MsgType {
    fn default() -> Self {
        MsgType::None
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct MsgDahai {
    #[serde(rename = "type")]
    type_: String,
    actor: usize,
    pai: String,
    tsumogiri: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct MsgPon {
    #[serde(rename = "type")]
    type_: String,
    actor: usize,
    target: usize,
    pai: String,
    consumed: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MsgChi {
    #[serde(rename = "type")]
    type_: String,
    actor: usize,
    target: usize,
    pai: String,
    consumed: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MsgKakan {
    #[serde(rename = "type")]
    type_: String,
    actor: usize,
    pai: String,
    consumed: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MsgDaiminkan {
    #[serde(rename = "type")]
    type_: String,
    actor: usize,
    target: usize,
    pai: String,
    consumed: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MsgAnkan {
    #[serde(rename = "type")]
    type_: String,
    actor: usize,
    consumed: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MsgReach {
    #[serde(rename = "type")]
    type_: String,
    actor: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct MsgHora {
    #[serde(rename = "type")]
    type_: String,
    actor: usize,
    target: usize,
    pai: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MsgRyukyoku {
    #[serde(rename = "type")]
    type_: String,
    actor: usize,
    reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MsgNone {
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Debug, Default)]
struct ClientMessage {
    type_: MsgType,
    dahai: Option<MsgDahai>,
    chi: Option<MsgChi>,
    pon: Option<MsgPon>,
    kakan: Option<MsgKakan>,
    daiminkan: Option<MsgDaiminkan>,
    ankan: Option<MsgAnkan>,
    reach: Option<MsgReach>,
    hora: Option<MsgHora>,
    ryukyoku: Option<MsgRyukyoku>,
    none: Option<MsgNone>,
}

impl ClientMessage {
    fn from_value(v: Value) -> Result<ClientMessage> {
        use serde_json::from_value;
        let type_ = v["type"]
            .as_str()
            .ok_or(serde::de::Error::missing_field("type"))?;

        let mut res = ClientMessage::default();
        match type_ {
            "dahai" => {
                res.type_ = MsgType::Dahai;
                res.dahai = from_value(v)?;
            }
            "chi" => {
                res.type_ = MsgType::Chi;
                res.chi = from_value(v)?;
                res.chi.as_mut().unwrap().consumed.sort();
            }
            "pon" => {
                res.type_ = MsgType::Pon;
                res.pon = from_value(v)?;
                res.pon.as_mut().unwrap().consumed.sort();
            }
            "kakan" => {
                res.type_ = MsgType::Kakan;
                res.kakan = from_value(v)?;
            }
            "daiminkan" => {
                res.type_ = MsgType::Daiminkan;
                res.daiminkan = from_value(v)?;
                res.daiminkan.as_mut().unwrap().consumed.sort();
            }
            "ankan" => {
                res.type_ = MsgType::Ankan;
                res.ankan = from_value(v)?;
                res.ankan.as_mut().unwrap().consumed.sort();
            }
            "reach" => {
                res.type_ = MsgType::Reach;
                res.reach = from_value(v)?;
            }
            "hora" => {
                res.type_ = MsgType::Hora;
                res.hora = from_value(v)?;
            }
            "ryukyoku" => {
                res.type_ = MsgType::Ryukyoku;
                res.ryukyoku = from_value(v)?;
            }
            "none" => {
                res.type_ = MsgType::None;
                res.none = from_value(v)?;
            }
            t => {
                return Err(serde::de::Error::invalid_value(
                    serde::de::Unexpected::Str(t),
                    &"type value",
                ))
            }
        }
        Ok(res)
    }
}

#[test]
fn test_mjai_message() {
    let dahai = r#"{"type":"dahai","actor":0,"pai":"6s","tsumogiri":false}"#;
    let chi = r#"{"type":"chi","actor":0,"target":3,"pai":"4p","consumed":["5p","6p"]}"#;
    let pon = r#"{"type":"pon","actor":0,"target":1,"pai":"5sr","consumed":["5s","5s"]}"#;
    let kakan = r#"{"type":"kakan","actor":0,"pai":"6m","consumed":["6m","6m","6m"]}"#;
    let daiminkan =
        r#"{"type":"daiminkan","actor":3,"target":1,"pai":"5m","consumed":["5m","5m","5mr"]}"#;
    let ankan = r#"{"type":"ankan","actor":1,"consumed":["N","N","N","N"]}"#;
    let reach = r#"{"type":"reach","actor":1}"#;
    let hora = r#"{"type":"hora","actor":1,"target":0,"pai":"7s"}"#;
    let none = r#"{"type":"none"}"#;
    let msgs = [dahai, chi, pon, kakan, daiminkan, ankan, reach, hora, none];

    for &msg in &msgs {
        let d = ClientMessage::from_value(serde_json::from_str(msg).unwrap()).unwrap();
        println!("{}, {:?}", msg, d);
    }
}
