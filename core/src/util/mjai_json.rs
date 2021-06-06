use serde::{Deserialize, Serialize};
use serde_json::{json, Result, Value};

use crate::hand::evaluate::WinContext;
use crate::model::*;

use PlayerOperationType::*;

// Mjai Message ===============================================================
// id: 自分の座席
// seat: 行動を行ったプレイヤーの座席
// target: 行動の対象となるプレイヤー(ロン, チー, ポン, 槓など)

pub fn mjai_start_game(id: Seat) -> Value {
    json!({
        "type":"start_game",
        "id": id,
        "names":["Player0","Player1","Player2","Player3"],
    })
}

pub fn mjai_start_kyoku(
    id: Seat,
    round: usize,
    kyoku: usize,
    honba: usize,
    kyotaku: usize,
    doras: &Vec<Tile>,
    player_hands: &[Vec<Tile>; SEAT],
) -> Value {
    let wind = ["E", "S", "W", "N"];
    let hands = create_tehais(player_hands, id);

    assert!(doras.len() == 1);
    let dora_marker = to_mjai_tile(doras[0]);

    json!({
        "type":"start_kyoku",
        "bakaze": wind[round],
        "kyoku": kyoku + 1,
        "honba": honba,
        "kyotaku": kyotaku,
        "dora_marker": dora_marker,
        "tehais": hands,
    })
}

pub fn mjai_tsumo(id: Seat, seat: Seat, tile: Tile) -> Value {
    let t = if id == seat {
        to_mjai_tile(tile)
    } else {
        "?".to_string()
    };
    json!({
        "type": "tsumo",
        "actor": seat,
        "pai": t,
    })
}

pub fn mjai_dahai(seat: Seat, tile: Tile, is_drawn: bool) -> Value {
    json!({
        "type": "dahai",
        "actor": seat,
        "pai": to_mjai_tile(tile),
        "tsumogiri": is_drawn,
    })
}

pub fn mjai_reach(seat: Seat) -> Value {
    json!({
        "type": "reach",
        "actor": seat,
    })
}

pub fn mjai_reach_accepted(seat: Seat, scores: [Score; SEAT]) -> Value {
    let mut deltas = [0, 0, 0, 0];
    deltas[seat] = -1000;
    json!({
        "type": "reach_accepted",
        "actor": seat,
        "deltas": deltas,
        "scores": scores,
    })
}

pub fn mjai_chiponkan(
    seat: Seat,
    meld_type: MeldType,
    tiles: &Vec<Tile>,
    froms: &Vec<Seat>,
) -> Value {
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
    json!({
        "type": type_,
        "actor": seat,
        "pai": pai,
        "target": target,
        "consumed": consumed,
    })
}

pub fn mjai_ankankakan(seat: Seat, meld_type: MeldType, tile: Tile, tiles: &Vec<Tile>) -> Value {
    match meld_type {
        MeldType::Ankan => {
            let mut consumed = vec![];
            for &t in tiles.iter() {
                consumed.push(to_mjai_tile(t))
            }
            json!({
                "type": "ankan",
                "actor": seat,
                "consumed": consumed,
            })
        }
        MeldType::Kakan => {
            let mut pai = "".to_string();
            let mut consumed = vec![];
            for &t in tiles.iter() {
                if pai == "" && t == tile {
                    pai = to_mjai_tile(t);
                } else {
                    consumed.push(to_mjai_tile(t))
                }
            }
            assert!(pai != "");

            json!({
                "type": "kakan",
                "actor": seat,
                "pai": pai,
                "consumed": consumed,
            })
        }
        _ => panic!(),
    }
}

pub fn mjai_dora(tile: Tile) -> Value {
    json!({
        "type": "dora",
        "dora_marker": to_mjai_tile(tile),
    })
}

pub fn mjai_hora(
    seat: Seat,
    target: Seat,
    tile: Tile,
    ura_doras: &Vec<Tile>,
    context: &WinContext,
    deltas: &[Score; SEAT],
    scores: &[Score; SEAT],
) -> Value {
    let ura: Vec<String> = ura_doras.iter().map(|&t| to_mjai_tile(t)).collect();
    json!({
        "type": "hora",
        "actor": seat,
        "target": target,
        "pai": to_mjai_tile(tile),
        "uradora_markers": ura,
        "hora_tehais": [], // TODO
        "yakus": [], // TODO
        "fu": context.fu,
        "fan": context.fan_mag,
        "hora_points": context.pay_scores.0,
        "deltas": deltas,
        "scores": scores,
    })
}

pub fn mjai_ryukyoku(
    draw_type: DrawType,
    is_ready: &[bool; SEAT],
    deltas: &[Score; SEAT],
    scores: &[Score; SEAT],
) -> Value {
    let type_ = match draw_type {
        DrawType::Kouhaiheikyoku => "fanpai",
        _ => "",
    };
    json!({
        "type": "ryukyoku",
        "reason": type_, // TODO
        "tehais": [], // TODO
        "tenpais": is_ready,
        "deltas": deltas,
        "scores": scores,
    })
}

pub fn mjai_end_game(scores: &[Score; SEAT]) -> Value {
    json!({
        "type": "end_game",
        "scores": scores,
    })
}

// Mjai Action ================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionDahai {
    #[serde(rename = "type")]
    type_: String,
    actor: Seat,
    pai: String,
    tsumogiri: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionPon {
    #[serde(rename = "type")]
    type_: String,
    actor: Seat,
    target: Seat,
    pai: String,
    consumed: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionChi {
    #[serde(rename = "type")]
    type_: String,
    actor: Seat,
    target: Seat,
    pai: String,
    consumed: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionKakan {
    #[serde(rename = "type")]
    type_: String,
    actor: Seat,
    pai: String,
    consumed: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionDaiminkan {
    #[serde(rename = "type")]
    type_: String,
    actor: Seat,
    target: Seat,
    pai: String,
    consumed: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionAnkan {
    #[serde(rename = "type")]
    type_: String,
    actor: Seat,
    consumed: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionReach {
    #[serde(rename = "type")]
    type_: String,
    actor: Seat,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionHora {
    #[serde(rename = "type")]
    type_: String,
    actor: Seat,
    target: Seat,
    pai: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionRyukyoku {
    #[serde(rename = "type")]
    type_: String,
    actor: Seat,
    reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionNone {
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MjaiAction {
    Dahai(ActionDahai),
    Pon(ActionPon),
    Chi(ActionChi),
    Kakan(ActionKakan),
    Daiminkan(ActionDaiminkan),
    Ankan(ActionAnkan),
    Reach(ActionReach),
    Hora(ActionHora),
    Ryukyoku(ActionRyukyoku),
    None(ActionNone),
}

impl MjaiAction {
    pub fn from_value(v: Value) -> Result<MjaiAction> {
        use serde_json::from_value;
        let type_ = v["type"]
            .as_str()
            .ok_or(serde::de::Error::missing_field("type"))?;

        Ok(match type_ {
            "dahai" => MjaiAction::Dahai(from_value(v)?),
            "chi" => {
                let mut act: ActionChi = from_value(v)?;
                act.consumed.sort();
                MjaiAction::Chi(act)
            }
            "pon" => {
                let mut act: ActionPon = from_value(v)?;
                act.consumed.sort();
                MjaiAction::Pon(act)
            }
            "kakan" => {
                let mut act: ActionKakan = from_value(v)?;
                act.consumed.sort();
                MjaiAction::Kakan(act)
            }
            "daiminkan" => {
                let mut act: ActionDaiminkan = from_value(v)?;
                act.consumed.sort();
                MjaiAction::Daiminkan(act)
            }
            "ankan" => {
                let mut act: ActionAnkan = from_value(v)?;
                act.consumed.sort();
                MjaiAction::Ankan(act)
            }
            "reach" => MjaiAction::Reach(from_value(v)?),
            "hora" => MjaiAction::Hora(from_value(v)?),
            "ryukyoku" => MjaiAction::Ryukyoku(from_value(v)?),
            "none" => MjaiAction::None(from_value(v)?),
            t => {
                return Err(serde::de::Error::invalid_value(
                    serde::de::Unexpected::Str(t),
                    &"type value",
                ))
            }
        })
    }

    pub fn to_value(&self) -> Value {
        match self {
            MjaiAction::Dahai(act) => json!(act),
            MjaiAction::Chi(act) => json!(act),
            MjaiAction::Pon(act) => json!(act),
            MjaiAction::Kakan(act) => json!(act),
            MjaiAction::Daiminkan(act) => json!(act),
            MjaiAction::Ankan(act) => json!(act),
            MjaiAction::Reach(act) => json!(act),
            MjaiAction::Hora(act) => json!(act),
            MjaiAction::Ryukyoku(act) => json!(act),
            MjaiAction::None(act) => json!(act),
        }
    }

    pub fn from_operation(stage: &Stage, seat: Seat, op: &PlayerOperation) -> Option<MjaiAction> {
        let PlayerOperation(tp, cs) = op;
        Some(match tp {
            Nop => return None,
            Discard => return None,
            Ankan => MjaiAction::Ankan(ActionAnkan {
                type_: "ankan".to_string(),
                actor: seat,
                consumed: vec_to_mjai_tile(cs),
            }),
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

                MjaiAction::Kakan(ActionKakan {
                    type_: "kakan".to_string(),
                    actor: seat,
                    pai: to_mjai_tile(t),
                    consumed: comsumed,
                })
            }
            Riichi => return None,
            Tsumo => MjaiAction::Hora(ActionHora {
                type_: "hora".to_string(),
                actor: seat,
                target: seat,
                pai: to_mjai_tile(stage.players[seat].drawn.unwrap()),
            }),
            Kyushukyuhai => MjaiAction::Ryukyoku(ActionRyukyoku {
                type_: "ryukyoku".to_string(),
                actor: seat,
                reason: "kyushukyuhai".to_string(),
            }),
            Kita => panic!(),
            Chii => {
                let (target_seat, _, target_tile) = stage.last_tile.unwrap();
                MjaiAction::Chi(ActionChi {
                    type_: "chi".to_string(),
                    actor: seat,
                    target: target_seat,
                    pai: to_mjai_tile(target_tile),
                    consumed: vec_to_mjai_tile(cs),
                })
            }
            Pon => {
                let (target_seat, _, target_tile) = stage.last_tile.unwrap();
                MjaiAction::Pon(ActionPon {
                    type_: "pon".to_string(),
                    actor: seat,
                    target: target_seat,
                    pai: to_mjai_tile(target_tile),
                    consumed: vec_to_mjai_tile(cs),
                })
            }
            Minkan => {
                let (target_seat, _, target_tile) = stage.last_tile.unwrap();
                MjaiAction::Daiminkan(ActionDaiminkan {
                    type_: "daiminkan".to_string(),
                    actor: seat,
                    target: target_seat,
                    pai: to_mjai_tile(target_tile),
                    consumed: vec_to_mjai_tile(cs),
                })
            }
            Ron => {
                let lt = stage.last_tile.unwrap();
                MjaiAction::Hora(ActionHora {
                    type_: "hora".to_string(),
                    actor: seat,
                    target: lt.0,
                    pai: to_mjai_tile(lt.2),
                })
            }
        })
    }

    pub fn to_operation(&self, is_turn: bool) -> PlayerOperation {
        match self {
            MjaiAction::Dahai(act) => {
                if act.tsumogiri {
                    Op::nop()
                } else {
                    Op::discard(from_mjai_tile(&act.pai))
                }
            }
            MjaiAction::Chi(act) => Op::chii(vec_from_mjai_tile(&act.consumed)),
            MjaiAction::Pon(act) => Op::pon(vec_from_mjai_tile(&act.consumed)),
            MjaiAction::Kakan(act) => Op::kakan(from_mjai_tile(&act.pai)),
            MjaiAction::Daiminkan(act) => Op::minkan(vec_from_mjai_tile(&act.consumed)),
            MjaiAction::Ankan(act) => Op::ankan(vec_from_mjai_tile(&act.consumed)),
            MjaiAction::Reach(_) => panic!(),
            MjaiAction::Hora(_) => {
                if is_turn {
                    Op::tsumo()
                } else {
                    Op::ron()
                }
            }
            MjaiAction::Ryukyoku(_) => Op::kyushukyuhai(),
            MjaiAction::None(_) => Op::nop(),
        }
    }

    pub fn from_value_to_operation(v: Value, is_turn: bool) -> Result<PlayerOperation> {
        Self::from_value(v).and_then(|cmsg| Ok(cmsg.to_operation(is_turn)))
    }

    pub fn from_operation_to_value(
        stage: &Stage,
        seat: Seat,
        op: &PlayerOperation,
    ) -> Option<Value> {
        Self::from_operation(stage, seat, op).and_then(|cmsg| Some(cmsg.to_value()))
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
            } as Type;
            let mut ni = (sym[0] - b'0') as Tnum;
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
        let d = MjaiAction::from_value(serde_json::from_str(msg).unwrap()).unwrap();
        println!("{}", d.to_value());
    }
}
