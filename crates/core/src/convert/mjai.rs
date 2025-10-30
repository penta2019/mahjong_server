use serde::{Deserialize, Serialize};

use crate::model::*;

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
        kyoku_first: usize, // 0: 4人南, 4: 4人東 (EventNew.modeとは割当が異なることに注意)
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
        actor: Seat,
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
        reason: String,
        tehais: Vec<Vec<String>>,
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
            protocol: "mjsonp".into(),
            protocol_version: 3,
        }
    }

    pub fn start_game(id: Seat, round: usize) -> Self {
        let kyoku_first = match round {
            1 => 4, // 4人東
            2 => 0, // 4人南
            _ => 4, // 不明な場合は4人東にしておく
        };
        Self::StartGame {
            id,
            names: [
                "Player0".into(),
                "Player1".into(),
                "Player2".into(),
                "Player3".into(),
            ],
            kyoku_first,
            aka_flag: true,
        }
    }

    pub fn start_kyoku(
        id: Seat,
        round: usize,
        dealer: Seat,
        honba: usize,
        riichi_sticks: usize,
        doras: &[Tile],
        hands: &[Vec<Tile>; SEAT],
        scores: &[Score; SEAT],
    ) -> Self {
        assert!(doras.len() == 1);
        let wind = ["E", "S", "W", "N"];
        Self::StartKyoku {
            bakaze: wind[round].into(),
            kyoku: dealer + 1,
            honba,
            kyotaku: riichi_sticks,
            oya: dealer,
            dora_marker: tile_to_mjai(doras[0]),
            tehais: create_tehais(hands, id),
            scores: *scores,
        }
    }

    pub fn tsumo(id: Seat, seat: Seat, tile: Tile) -> Self {
        let t = if id == seat {
            tile_to_mjai(tile)
        } else {
            "?".into()
        };
        Self::Tsumo {
            actor: seat,
            pai: t,
        }
    }

    pub fn dahai(seat: Seat, tile: Tile, is_drawn: bool) -> Self {
        Self::Dahai {
            actor: seat,
            pai: tile_to_mjai(tile),
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
            deltas,
            scores: *scores,
        }
    }

    pub fn chi(seat: Seat, consumed: &[Tile], tile: Tile, target: Seat) -> Self {
        Self::Chi {
            actor: seat,
            pai: tile_to_mjai(tile),
            consumed: tiles_to_mjai(consumed),
            target,
        }
    }

    pub fn pon(seat: Seat, consumed: &[Tile], tile: Tile, target: Seat) -> Self {
        Self::Pon {
            actor: seat,
            pai: tile_to_mjai(tile),
            consumed: tiles_to_mjai(consumed),
            target,
        }
    }

    pub fn daiminkan(seat: Seat, consumed: &[Tile], tile: Tile, target: Seat) -> Self {
        Self::Daiminkan {
            actor: seat,
            pai: tile_to_mjai(tile),
            consumed: tiles_to_mjai(consumed),
            target,
        }
    }

    pub fn ankan(seat: Seat, consumed: &[Tile]) -> Self {
        Self::Ankan {
            actor: seat,
            consumed: tiles_to_mjai(consumed),
        }
    }

    pub fn kakan(seat: Seat, consumed: &[Tile], pon_tiles: &[Tile]) -> Self {
        Self::Kakan {
            actor: seat,
            pai: tile_to_mjai(consumed[0]),
            consumed: tiles_to_mjai(pon_tiles),
        }
    }

    pub fn dora(tile: Tile) -> Self {
        Self::Dora {
            dora_marker: tile_to_mjai(tile),
        }
    }

    pub fn hora(
        seat: Seat,
        target: Seat,
        tile: Tile,
        ura_doras: &[Tile],
        context: &ScoreContext,
        deltas: &[Point; SEAT],
        scores: &[Score; SEAT],
    ) -> Self {
        Self::Hora {
            actor: seat,
            target,
            pai: tile_to_mjai(tile),
            uradora_markers: tiles_to_mjai(ura_doras),
            hora_tehais: vec![], // TODO
            yakus: vec![],       // TODO
            fu: context.fu,
            fan: context.fan,
            hora_points: context.points.0,
            deltas: *deltas,
            scores: *scores,
        }
    }

    pub fn ryukyoku(
        type_: DrawType,
        is_tenpai: &[bool; SEAT],
        deltas: &[Point; SEAT],
        scores: &[Score; SEAT],
    ) -> Self {
        let reason = match type_ {
            DrawType::Kouhaiheikyoku => "fanpai",
            _ => "",
        };
        Self::Ryukyoku {
            reason: reason.into(), // TODO
            tehais: vec![],        // TODO
            tenpais: *is_tenpai,
            deltas: *deltas,
            scores: *scores,
        }
    }

    pub fn end_kyoku() -> Self {
        Self::EndKyoku {}
    }

    pub fn end_game(scores: &[Score; SEAT]) -> Self {
        Self::EndGame { scores: *scores }
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
    pub fn from_action(stg: &Stage, seat: Seat, act: &Action) -> Option<Self> {
        let tp = act.ty;
        let cs = &act.tiles;
        Some(match tp {
            ActionType::Nop => return None,
            ActionType::Discard => return None,
            ActionType::Ankan => Self::Ankan {
                actor: seat,
                consumed: tiles_to_mjai(cs),
            },
            ActionType::Kakan => {
                let t = cs[0];
                let comsumed = tiles_to_mjai(&if t.1 == 0 {
                    // 赤5
                    let t2 = Tile(t.0, 5);
                    vec![t2, t2, t2]
                } else if t.is_suit() && t.1 == 5 {
                    // 通常5
                    vec![Tile(t.0, 0), t, t]
                } else {
                    vec![t, t, t]
                });

                Self::Kakan {
                    actor: seat,
                    pai: tile_to_mjai(t),
                    consumed: comsumed,
                }
            }
            ActionType::Riichi => return None,
            ActionType::Tsumo => Self::Hora {
                actor: seat,
                target: seat,
                pai: tile_to_mjai(stg.players[seat].drawn.unwrap()),
            },
            ActionType::Kyushukyuhai => Self::Ryukyoku {
                actor: seat,
                reason: "kyushukyuhai".into(),
            },
            ActionType::Nukidora => panic!(),
            ActionType::Chi => {
                let (target_seat, _, target_tile) = stg.last_tile.unwrap();
                Self::Chi {
                    actor: seat,
                    target: target_seat,
                    pai: tile_to_mjai(target_tile),
                    consumed: tiles_to_mjai(cs),
                }
            }
            ActionType::Pon => {
                let (target_seat, _, target_tile) = stg.last_tile.unwrap();
                Self::Pon {
                    actor: seat,
                    target: target_seat,
                    pai: tile_to_mjai(target_tile),
                    consumed: tiles_to_mjai(cs),
                }
            }
            ActionType::Minkan => {
                let (target_seat, _, target_tile) = stg.last_tile.unwrap();
                Self::Daiminkan {
                    actor: seat,
                    target: target_seat,
                    pai: tile_to_mjai(target_tile),
                    consumed: tiles_to_mjai(cs),
                }
            }
            ActionType::Ron => {
                let lt = stg.last_tile.unwrap();
                Self::Hora {
                    actor: seat,
                    target: lt.0,
                    pai: tile_to_mjai(lt.2),
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
                    Action::discard(tile_from_mjai(pai))
                }
            }
            Self::Chi { consumed, .. } => Action::chi(tiles_from_mjai(consumed)),
            Self::Pon { consumed, .. } => Action::pon(tiles_from_mjai(consumed)),
            Self::Kakan { pai, .. } => Action::kakan(tile_from_mjai(pai)),
            Self::Daiminkan { consumed, .. } => Action::minkan(tiles_from_mjai(consumed)),
            Self::Ankan { consumed, .. } => Action::ankan(tiles_from_mjai(consumed)),
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
pub fn tile_to_mjai(t: Tile) -> String {
    if t.is_hornor() {
        assert!(WE <= t.1 && t.1 <= DR);
        let hornor = ["", "E", "S", "W", "N", "P", "F", "C"];
        hornor[t.1].into()
    } else {
        let tile_type = ["m", "p", "s"];
        format!(
            "{}{}{}",
            t.to_normal().1,
            tile_type[t.0],
            if t.1 == 0 { "r" } else { "" }
        )
    }
}

pub fn tile_from_mjai(sym: &str) -> Tile {
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

fn tiles_to_mjai(v: &[Tile]) -> Vec<String> {
    v.iter().map(|&t| tile_to_mjai(t)).collect()
}

fn tiles_from_mjai(v: &[String]) -> Vec<Tile> {
    let mut v2: Vec<Tile> = v.iter().map(|t| tile_from_mjai(t)).collect();
    v2.sort();
    v2
}

fn create_tehais(hands: &[Vec<Tile>; SEAT], seat: usize) -> [Vec<String>; SEAT] {
    let mut mjai_hands = [vec![], vec![], vec![], vec![]];
    for (seat2, hand) in hands.iter().enumerate() {
        let mut mjai_hand = vec![];
        for &t in hand {
            if seat == seat2 {
                mjai_hand.push(tile_to_mjai(t));
            } else {
                mjai_hand.push("?".into());
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
        let act: MjaiAction = serde_json::from_str(msg).unwrap();
        println!("{:?}", act);
    }
}
