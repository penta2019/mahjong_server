use serde::{Deserialize, Serialize};

use crate::hand::WinContext;
use crate::model::*;

use ActionType::*;

// [MjaiEvent]
// サーバ側から送信する情報
// id: 自分の座席
// seat: 行動を行ったプレイヤーの座席
// target: 行動の対象となるプレイヤー(ロン, チー, ポン, 槓など)
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum MjaiEvent {
    Hello {
        protocol: String,
        protocol_version: usize,
    },
    StartGame {
        id: Seat,
        names: [String; SEAT],
        kyoku_first: usize, // 0: 4人南, 4: 4人東 (EventRoundNew.modeとは割当が異なることに注意)
        aka_flag: bool,     // true: 赤ドラあり
    },
    StartKyoku {
        bakaze: String,
        dora_marker: String,
        kyoku: usize, // counts from 1
        honba: usize,
        kyotaku: usize,
        oya: Seat,
        tehais: [Vec<String>; SEAT],
        scores: [Score; SEAT],
    },
    Tsumo {
        actor: Seat,
        pai: String,
    },
    Dahai {
        actor: usize,
        pai: String,
        tsumogiri: bool,
    },
    Chi {
        actor: Seat,
        target: Seat,
        pai: String,
        consumed: Vec<String>,
    },
    Pon {
        actor: Seat,
        target: Seat,
        pai: String,
        consumed: Vec<String>,
    },
    Daiminkan {
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
    Ankan {
        actor: Seat,
        consumed: Vec<String>,
    },
    Dora {
        dora_marker: String,
    },
    Reach {
        actor: Seat,
    },
    ReachAccepted {
        actor: Seat,
        deltas: [Point; SEAT],
        scores: [Score; SEAT],
    },
    Hora {
        actor: Seat,
        target: Seat,
        pai: String,
        uradora_markers: Vec<String>,
        hora_tehais: Vec<String>,
        yakus: Vec<String>,
        fu: usize,
        fan: usize,
        hora_points: Point,
        deltas: [Point; SEAT],
        scores: [Score; SEAT],
    },
    Ryukyoku {
        reason: String,      // TODO
        tehais: Vec<String>, // TODO
        tenpais: [bool; SEAT],
        deltas: [Point; SEAT],
        scores: [Score; SEAT],
    },
    EndKyoku {},
    EndGame {
        scores: [Score; SEAT],
    },
    None {},
}

impl MjaiEvent {
    pub fn hello() -> Self {
        Self::Hello {
            protocol: "mjsonp".to_string(),
            protocol_version: 3,
        }
    }

    pub fn start_game(id: Seat, mode: usize) -> Self {
        let kyoku_first = match mode {
            1 => 4, // 4人東
            2 => 0, // 4人南
            _ => 4, // 不明な場合は4人東にしておく
        };
        Self::StartGame {
            id: id,
            names: [
                "Player0".to_string(),
                "Player1".to_string(),
                "Player2".to_string(),
                "Player3".to_string(),
            ],
            kyoku_first: kyoku_first,
            aka_flag: true,
        }
    }

    pub fn start_kyoku(
        id: Seat,
        round: usize,
        kyoku: usize,
        honba: usize,
        kyotaku: usize,
        doras: &Vec<Tile>,
        hands: &[Vec<Tile>; SEAT],
        scores: &[Score; SEAT],
    ) -> Self {
        assert!(doras.len() == 1);
        let wind = ["E", "S", "W", "N"];
        Self::StartKyoku {
            bakaze: wind[round].to_string(),
            kyoku: kyoku + 1,
            honba: honba,
            kyotaku: kyotaku,
            oya: kyoku,
            dora_marker: to_mjai_tile(doras[0]),
            tehais: create_tehais(hands, id),
            scores: scores.clone(),
        }
    }

    pub fn tsumo(id: Seat, seat: Seat, tile: Tile) -> Self {
        let t = if id == seat {
            to_mjai_tile(tile)
        } else {
            "?".to_string()
        };
        Self::Tsumo {
            actor: seat,
            pai: t,
        }
    }

    pub fn dahai(seat: Seat, tile: Tile, is_drawn: bool) -> Self {
        Self::Dahai {
            actor: seat,
            pai: to_mjai_tile(tile),
            tsumogiri: is_drawn,
        }
    }

    pub fn reach(seat: Seat) -> Self {
        Self::Reach { actor: seat }
    }

    pub fn reach_accepted(seat: Seat, scores: &[Score; SEAT]) -> Self {
        let mut deltas = [0, 0, 0, 0];
        deltas[seat] = -1000;
        Self::ReachAccepted {
            actor: seat,
            deltas: deltas,
            scores: scores.clone(),
        }
    }

    pub fn chi(seat: Seat, consumed: &Vec<Tile>, tile: Tile, target: Seat) -> Self {
        Self::Chi {
            actor: seat,
            pai: to_mjai_tile(tile),
            consumed: vec_to_mjai_tile(consumed),
            target: target,
        }
    }

    pub fn pon(seat: Seat, consumed: &Vec<Tile>, tile: Tile, target: Seat) -> Self {
        Self::Pon {
            actor: seat,
            pai: to_mjai_tile(tile),
            consumed: vec_to_mjai_tile(consumed),
            target: target,
        }
    }

    pub fn daiminkan(seat: Seat, consumed: &Vec<Tile>, tile: Tile, target: Seat) -> Self {
        Self::Daiminkan {
            actor: seat,
            pai: to_mjai_tile(tile),
            consumed: vec_to_mjai_tile(consumed),
            target: target,
        }
    }

    pub fn ankan(seat: Seat, consumed: &Vec<Tile>) -> Self {
        Self::Ankan {
            actor: seat,
            consumed: vec_to_mjai_tile(consumed),
        }
    }

    pub fn kakan(seat: Seat, consumed: &Vec<Tile>, pon_tiles: &Vec<Tile>) -> Self {
        Self::Kakan {
            actor: seat,
            pai: to_mjai_tile(consumed[0]),
            consumed: vec_to_mjai_tile(pon_tiles),
        }
    }

    pub fn dora(tile: Tile) -> Self {
        Self::Dora {
            dora_marker: to_mjai_tile(tile),
        }
    }

    pub fn hora(
        seat: Seat,
        target: Seat,
        tile: Tile,
        ura_doras: &Vec<Tile>,
        context: &WinContext,
        deltas: &[Point; SEAT],
        scores: &[Score; SEAT],
    ) -> Self {
        let ura: Vec<String> = ura_doras.iter().map(|&t| to_mjai_tile(t)).collect();
        Self::Hora {
            actor: seat,
            target: target,
            pai: to_mjai_tile(tile),
            uradora_markers: ura,
            hora_tehais: vec![], // TODO
            yakus: vec![],       // TODO
            fu: context.fu,
            fan: context.fan,
            hora_points: context.points.0,
            deltas: deltas.clone(),
            scores: scores.clone(),
        }
    }

    pub fn ryukyoku(
        draw_type: DrawType,
        is_tenpai: &[bool; SEAT],
        deltas: &[Point; SEAT],
        scores: &[Score; SEAT],
    ) -> Self {
        let reason = match draw_type {
            DrawType::Kouhaiheikyoku => "fanpai",
            _ => "",
        };
        Self::Ryukyoku {
            reason: reason.to_string(), // TODO
            tehais: vec![],             // TODO
            tenpais: is_tenpai.clone(),
            deltas: deltas.clone(),
            scores: scores.clone(),
        }
    }

    // pub fn end_kyoku(deltas: &[Score; SEAT]) {}

    pub fn end_game(scores: &[Score; SEAT]) -> Self {
        Self::EndGame {
            scores: scores.clone(),
        }
    }
}

// [MjaiAction]
// MjaiEvent内のpossible_actionの中身とクライアント側の応答
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum MjaiAction {
    Join {
        name: String,
        room: String,
    },
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
    None {},
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
            Self::Join { .. } => panic!(),
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
            Self::None {} => Action::nop(),
        }
    }
}

// [Utility]
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
    let mut v2: Vec<Tile> = v.iter().map(|t| from_mjai_tile(t)).collect();
    v2.sort();
    v2
}

fn create_tehais(hands: &[Vec<Tile>; SEAT], seat: usize) -> [Vec<String>; SEAT] {
    let mut mjai_hands = [vec![], vec![], vec![], vec![]];
    for (seat2, hand) in hands.iter().enumerate() {
        let mut mjai_hand = vec![];
        for &t in hand {
            if seat == seat2 {
                mjai_hand.push(to_mjai_tile(t));
            } else {
                mjai_hand.push("?".to_string());
            }
        }
        mjai_hands[seat2] = mjai_hand;
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
        let a: MjaiAction = serde_json::from_str(msg).unwrap();
        println!("{:?}", a);
    }
}
