use serde::{Deserialize, Serialize};
use serde_json::{json, Result, Value};

use crate::model::*;

use PlayerOperationType::*;

#[derive(Debug)]
pub enum MsgType {
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
pub struct ClientMessage {
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
    pub fn from_value(v: Value) -> Result<ClientMessage> {
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

    pub fn to_value(&self) -> Value {
        match self.type_ {
            MsgType::Dahai => json!(self.dahai),
            MsgType::Chi => json!(self.chi),
            MsgType::Pon => json!(self.pon),
            MsgType::Kakan => json!(self.kakan),
            MsgType::Daiminkan => json!(self.daiminkan),
            MsgType::Ankan => json!(self.ankan),
            MsgType::Reach => json!(self.reach),
            MsgType::Hora => json!(self.hora),
            MsgType::Ryukyoku => json!(self.ryukyoku),
            MsgType::None => json!(self.none),
        }
    }

    pub fn from_operation(stg: &Stage, seat: Seat, op: &PlayerOperation) -> Option<ClientMessage> {
        let mut res = ClientMessage::default();
        let PlayerOperation(tp, cs) = op;
        match tp {
            Nop => return None,
            Discard => return None,
            Ankan => {
                res.type_ = MsgType::Ankan;
                res.ankan = Some(MsgAnkan {
                    type_: "ankan".to_string(),
                    actor: seat,
                    consumed: vec_to_mjai_tile(cs),
                })
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
                res.type_ = MsgType::Kakan;
                res.kakan = Some(MsgKakan {
                    type_: "kakan".to_string(),
                    actor: seat,
                    pai: to_mjai_tile(t),
                    consumed: comsumed,
                });
            }
            Riichi => return None,
            Tsumo => {
                res.type_ = MsgType::Hora;
                res.hora = Some(MsgHora {
                    type_: "hora".to_string(),
                    actor: seat,
                    target: seat,
                    pai: to_mjai_tile(stg.players[seat].drawn.unwrap()),
                });
            }
            Kyushukyuhai => {
                res.type_ = MsgType::Ryukyoku;
                res.ryukyoku = Some(MsgRyukyoku {
                    type_: "ryukyoku".to_string(),
                    actor: seat,
                    reason: "kyushukyuhai".to_string(),
                });
            }
            Kita => {
                panic!()
            }
            Chii => {
                let (target_seat, _, target_tile) = stg.last_tile.unwrap();
                res.type_ = MsgType::Chi;
                res.chi = Some(MsgChi {
                    type_: "chi".to_string(),
                    actor: seat,
                    target: target_seat,
                    pai: to_mjai_tile(target_tile),
                    consumed: vec_to_mjai_tile(cs),
                });
            }
            Pon => {
                let (target_seat, _, target_tile) = stg.last_tile.unwrap();
                res.type_ = MsgType::Pon;
                res.pon = Some(MsgPon {
                    type_: "pon".to_string(),
                    actor: seat,
                    target: target_seat,
                    pai: to_mjai_tile(target_tile),
                    consumed: vec_to_mjai_tile(cs),
                });
            }
            Minkan => {
                let (target_seat, _, target_tile) = stg.last_tile.unwrap();
                res.type_ = MsgType::Daiminkan;
                res.daiminkan = Some(MsgDaiminkan {
                    type_: "daiminkan".to_string(),
                    actor: seat,
                    target: target_seat,
                    pai: to_mjai_tile(target_tile),
                    consumed: vec_to_mjai_tile(cs),
                });
            }
            Ron => {
                let lt = stg.last_tile.unwrap();
                res.type_ = MsgType::Hora;
                res.hora = Some(MsgHora {
                    type_: "hora".to_string(),
                    actor: seat,
                    target: lt.0,
                    pai: to_mjai_tile(lt.2),
                });
            }
        }
        Some(res)
    }

    pub fn to_operation(&self, is_turn: bool) -> PlayerOperation {
        match self.type_ {
            MsgType::Dahai => {
                let m = self.dahai.as_ref().unwrap();
                if m.tsumogiri {
                    Op::nop()
                } else {
                    Op::discard(from_mjai_tile(&m.pai))
                }
            }
            MsgType::Chi => {
                let m = self.chi.as_ref().unwrap();
                Op::chii(vec_from_mjai_tile(&m.consumed))
            }
            MsgType::Pon => {
                let m = self.pon.as_ref().unwrap();
                Op::pon(vec_from_mjai_tile(&m.consumed))
            }
            MsgType::Kakan => {
                let m = self.kakan.as_ref().unwrap();
                Op::kakan(from_mjai_tile(&m.pai))
            }
            MsgType::Daiminkan => {
                let m = self.daiminkan.as_ref().unwrap();
                Op::minkan(vec_from_mjai_tile(&m.consumed))
            }
            MsgType::Ankan => {
                let m = self.ankan.as_ref().unwrap();
                Op::ankan(vec_from_mjai_tile(&m.consumed))
            }
            MsgType::Reach => panic!(),
            MsgType::Hora => {
                if is_turn {
                    Op::tsumo()
                } else {
                    Op::ron()
                }
            }
            MsgType::Ryukyoku => Op::kyushukyuhai(),
            MsgType::None => Op::nop(),
        }
    }

    pub fn from_value_to_operation(v: Value, is_turn: bool) -> Result<PlayerOperation> {
        Self::from_value(v).and_then(|cmsg| Ok(cmsg.to_operation(is_turn)))
    }

    pub fn from_operation_to_value(stg: &Stage, seat: Seat, op: &PlayerOperation) -> Option<Value> {
        Self::from_operation(stg, seat, op).and_then(|cmsg| Some(cmsg.to_value()))
    }
}

pub fn to_mjai_tile(t: Tile) -> String {
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

pub fn from_mjai_tile(sym: &str) -> Tile {
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

pub fn vec_to_mjai_tile(v: &Vec<Tile>) -> Vec<String> {
    v.iter().map(|&t| to_mjai_tile(t)).collect()
}

pub fn vec_from_mjai_tile(v: &Vec<String>) -> Vec<Tile> {
    v.iter().map(|t| from_mjai_tile(t)).collect()
}

pub fn create_tehais(player_hands: &[Vec<Tile>; SEAT], seat: usize) -> Vec<Vec<String>> {
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
