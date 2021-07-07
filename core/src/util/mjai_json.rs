use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::hand::evaluate::WinContext;
use crate::model::*;

use ActionType::*;

// [Mjai Message]
// id: 自分の座席
// seat: 行動を行ったプレイヤーの座席
// target: 行動の対象となるプレイヤー(ロン, チー, ポン, 槓など)

pub fn mjai_hello() -> Value {
    json!({"type": "hello","protocol": "mjsonp","protocol_version": 3})
}

pub fn mjai_start_game(id: Seat) -> Value {
    json!({
        "type":"start_game",
        "id": id,
        "names":["Player0", "Player1", "Player2", "Player3"],
    })
}

pub fn mjai_start_kyoku(
    id: Seat,
    round: usize,
    kyoku: usize,
    honba: usize,
    kyotaku: usize,
    doras: &Vec<Tile>,
    hands: &[Vec<Tile>; SEAT],
) -> Value {
    let wind = ["E", "S", "W", "N"];
    let hands = create_tehais(hands, id);

    assert!(doras.len() == 1);
    let dora_marker = to_mjai_tile(doras[0]);

    json!({
        "type": "start_kyoku",
        "bakaze": wind[round],
        "kyoku": kyoku + 1,
        "honba": honba,
        "kyotaku": kyotaku,
        "oya": kyoku,
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
    consumed: &Vec<Tile>,
    tile: Tile,
    target: Seat,
) -> Value {
    let type_ = match meld_type {
        MeldType::Chi => "chi",
        MeldType::Pon => "pon",
        MeldType::Minkan => "daiminkan",
        _ => panic!(),
    };
    json!({
        "type": type_,
        "actor": seat,
        "pai": to_mjai_tile(tile),
        "consumed": vec_to_mjai_tile(consumed),
        "target": target,
    })
}

pub fn mjai_ankan(seat: Seat, consumed: &Vec<Tile>) -> Value {
    json!({
        "type": "ankan",
        "actor": seat,
        "consumed": vec_to_mjai_tile(consumed),
    })
}

pub fn mjai_kakan(seat: Seat, consumed: &Vec<Tile>, pon_tiles: &Vec<Tile>) -> Value {
    json!({
        "type": "kakan",
        "actor": seat,
        "pai": to_mjai_tile(consumed[0]),
        "consumed": vec_to_mjai_tile(pon_tiles),
    })
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
    deltas: &[Point; SEAT],
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
        "fan": context.fan,
        "hora_points": context.points.0,
        "deltas": deltas,
        "scores": scores,
    })
}

pub fn mjai_ryukyoku(
    draw_type: DrawType,
    is_tenpai: &[bool; SEAT],
    deltas: &[Point; SEAT],
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
        "tenpais": is_tenpai,
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum MjaiAction {
    Dahai {
        actor: Seat,
        pai: String,
        tsumogiri: bool,
    },
    Pon {
        actor: Seat,
        target: Seat,
        pai: String,
        consumed: Vec<String>,
    },
    Chi {
        actor: Seat,
        target: Seat,
        pai: String,
        consumed: Vec<String>,
    },
    Kakan {
        actor: Seat,
        pai: String,
        consumed: Vec<String>,
    },
    Daiminkan {
        actor: Seat,
        target: Seat,
        pai: String,
        consumed: Vec<String>,
    },
    Ankan {
        actor: Seat,
        consumed: Vec<String>,
    },
    Reach {
        actor: Seat,
    },
    Hora {
        actor: Seat,
        target: Seat,
        pai: String,
    },
    Ryukyoku {
        actor: Seat,
        reason: String,
    },
    None,
}

impl MjaiAction {
    pub fn from_action(stage: &Stage, seat: Seat, act: &Action) -> Option<Self> {
        let Action(tp, cs) = act;
        Some(match tp {
            Nop => return None,
            Discard => return None,
            Ankan => Self::Ankan {
                actor: seat,
                consumed: vec_to_mjai_tile(cs),
            },
            Kakan => {
                let t = act.1[0];
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

                Self::Kakan {
                    actor: seat,
                    pai: to_mjai_tile(t),
                    consumed: comsumed,
                }
            }
            Riichi => return None,
            Tsumo => Self::Hora {
                actor: seat,
                target: seat,
                pai: to_mjai_tile(stage.players[seat].drawn.unwrap()),
            },
            Kyushukyuhai => Self::Ryukyoku {
                actor: seat,
                reason: "kyushukyuhai".to_string(),
            },
            Kita => panic!(),
            Chi => {
                let (target_seat, _, target_tile) = stage.last_tile.unwrap();
                Self::Chi {
                    actor: seat,
                    target: target_seat,
                    pai: to_mjai_tile(target_tile),
                    consumed: vec_to_mjai_tile(cs),
                }
            }
            Pon => {
                let (target_seat, _, target_tile) = stage.last_tile.unwrap();
                Self::Pon {
                    actor: seat,
                    target: target_seat,
                    pai: to_mjai_tile(target_tile),
                    consumed: vec_to_mjai_tile(cs),
                }
            }
            Minkan => {
                let (target_seat, _, target_tile) = stage.last_tile.unwrap();
                Self::Daiminkan {
                    actor: seat,
                    target: target_seat,
                    pai: to_mjai_tile(target_tile),
                    consumed: vec_to_mjai_tile(cs),
                }
            }
            Ron => {
                let lt = stage.last_tile.unwrap();
                Self::Hora {
                    actor: seat,
                    target: lt.0,
                    pai: to_mjai_tile(lt.2),
                }
            }
        })
    }

    pub fn to_action(&self, is_turn: bool) -> Action {
        match self {
            Self::Dahai { pai, tsumogiri, .. } => {
                if *tsumogiri {
                    Action::nop()
                } else {
                    Action::discard(from_mjai_tile(pai))
                }
            }
            Self::Chi { consumed, .. } => Action::chi(vec_from_mjai_tile(consumed)),
            Self::Pon { consumed, .. } => Action::pon(vec_from_mjai_tile(consumed)),
            Self::Kakan { pai, .. } => Action::kakan(from_mjai_tile(pai)),
            Self::Daiminkan { consumed, .. } => Action::minkan(vec_from_mjai_tile(consumed)),
            Self::Ankan { consumed, .. } => Action::ankan(vec_from_mjai_tile(consumed)),
            Self::Reach { .. } => panic!(),
            Self::Hora { .. } => {
                if is_turn {
                    Action::tsumo()
                } else {
                    Action::ron()
                }
            }
            Self::Ryukyoku { .. } => Action::kyushukyuhai(),
            Self::None => Action::nop(),
        }
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
            t.to_normal().1,
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

fn create_tehais(hands: &[Vec<Tile>; SEAT], seat: usize) -> Vec<Vec<String>> {
    let mut mjai_hands = vec![];
    for (seat2, hand) in hands.iter().enumerate() {
        let mut mjai_hand = vec![];
        for &t in hand {
            if seat == seat2 {
                mjai_hand.push(to_mjai_tile(t));
            } else {
                mjai_hand.push("?".to_string());
            }
        }
        mjai_hands.push(mjai_hand);
    }
    mjai_hands
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
        // let d = MjaiAction::from_value(serde_json::from_str(msg).unwrap()).unwrap();
        let a: MjaiAction = serde_json::from_str(msg).unwrap();
        println!("{}", serde_json::to_string(&a).unwrap());
    }
}
