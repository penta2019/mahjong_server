use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::model::*;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TenhouLog {
    pub log: Vec<Value>,
    pub name: [String; SEAT],
    pub rule: TenhouRule,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ratingc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lobby: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dan: Option<[String; SEAT]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate: Option<[f32; SEAT]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sx: Option<[String; SEAT]>,
}

impl TenhouLog {
    pub fn new() -> Self {
        let mut log = Self::default();
        log.rule.disp = "東喰赤".to_string();
        log.rule.aka = 1;
        log
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TenhouRule {
    pub disp: String,
    pub aka: usize,
    pub aka51: usize,
    pub aka52: usize,
    pub aka53: usize,
}

#[derive(Debug, Default)]
struct TenhouPlayer {
    hand: Vec<i64>,       // 配牌13枚 (親番14枚目はツモ扱い)
    drawns: Vec<Value>,   // 他家から鳴きで得た配を含む
    discards: Vec<Value>, // 捨て牌(ツモ切りの情報を含む)
}

#[derive(Debug, Default)]
struct TenhouRound {
    dealer: Seat,
    honba_sticks: usize,
    riichi_sticks: usize,
    scores: [Score; SEAT],
    doras: Vec<i64>,
    ura_doras: Vec<i64>,
    players: [TenhouPlayer; SEAT],
    result: String,
    result_detail: Vec<Vec<Value>>,
}

impl TenhouRound {
    fn to_log(&self) -> Value {
        let mut v = vec![
            json!([self.dealer, self.honba_sticks, self.riichi_sticks]),
            json!(self.scores),
            json!(self.doras),
            json!(self.ura_doras),
        ];
        for p in &self.players {
            v.push(json!(p.hand));
            v.push(json!(p.drawns));
            v.push(json!(p.discards));
        }
        let mut result = vec![json!(self.result)];
        for d in &self.result_detail {
            result.push(json!(d));
        }
        v.push(json!(result));
        json!(v)
    }
}

// [TenhouSerializer]
#[derive(Debug)]
pub struct TenhouSerializer {
    log: TenhouLog,
    dealer: TenhouRound,
}

impl TenhouSerializer {
    pub fn new() -> Self {
        Self {
            log: TenhouLog::new(),
            dealer: TenhouRound::default(),
        }
    }

    pub fn push_event(&mut self, stg: &Stage, event: &Event) {
        let k = &mut self.dealer;
        match event {
            Event::Begin(_) => {}
            Event::New(ev) => {
                self.dealer = TenhouRound::default();
                let k = &mut self.dealer;
                k.dealer = ev.round * 4 + ev.dealer;
                k.honba_sticks = ev.honba_sticks;
                k.riichi_sticks = ev.riichi_sticks;
                k.doras = tiles_to_tenhou(&ev.doras);
                k.scores = ev.scores;
                for s in 0..SEAT {
                    k.players[s].hand = tiles_to_tenhou(&ev.hands[s]);
                }
            }
            Event::Deal(ev) => {
                k.players[ev.seat]
                    .drawns
                    .push(json!(tile_to_tenhou(ev.tile)));
            }
            Event::Discard(ev) => {
                let d = if ev.is_drawn {
                    60
                } else {
                    tile_to_tenhou(ev.tile)
                };
                let d = if ev.is_riichi {
                    json!(format!("r{}", d))
                } else {
                    json!(d)
                };
                k.players[ev.seat].discards.push(d);
            }
            Event::Meld(ev) => match ev.meld_type {
                MeldType::Chi | MeldType::Pon | MeldType::Minkan => {
                    let (seat, _, d) = stg.last_tile.unwrap();
                    let pos = 3 - (seat + SEAT - ev.seat) % SEAT;
                    let marker = match ev.meld_type {
                        MeldType::Chi => "c",
                        MeldType::Pon => "p",
                        MeldType::Minkan => "m",
                        _ => panic!(),
                    };
                    let mut meld: Vec<String> = ev
                        .consumed
                        .iter()
                        .map(|&t| tile_to_tenhou(t).to_string())
                        .collect();
                    meld.insert(pos, format!("{}{}", marker, tile_to_tenhou(d)));
                    k.players[ev.seat].drawns.push(json!(meld.concat()));
                }
                MeldType::Kakan => {
                    let t = ev.consumed[0];
                    let tn = t.to_normal();
                    for m in &stg.players[ev.seat].melds {
                        if m.tiles[0].to_normal() == tn {
                            let mut meld = String::new();
                            for i in 0..3 {
                                if m.froms[i] != ev.seat {
                                    meld += "k";
                                    meld += &tile_to_tenhou(t).to_string();
                                }
                                meld += &tile_to_tenhou(m.tiles[i]).to_string();
                            }
                            k.players[ev.seat].discards.push(json!(meld));
                        }
                    }
                }
                MeldType::Ankan => {
                    let mut meld: Vec<String> = ev
                        .consumed
                        .iter()
                        .map(|&t| tile_to_tenhou(t).to_string())
                        .collect();
                    meld.insert(3, "a".to_string());
                    k.players[ev.seat].discards.push(json!(meld.concat()));
                }
            },
            Event::Nukidora(_) => panic!(),
            Event::Dora(ev) => {
                k.doras.push(tile_to_tenhou(ev.tile));
            }
            Event::Win(ev) => {
                k.result = "和了".to_string();
                k.ura_doras = tiles_to_tenhou(&ev.ura_doras);
                for ctx in &ev.contexts {
                    let score_ctx = &ctx.score_context;
                    k.result_detail
                        .push(ctx.delta_scores.iter().map(|&p| json!(p)).collect());
                    // detail = [和了者, 放銃者, 責任払い?]
                    let mut detail = vec![json!(ctx.seat), json!(stg.turn), json!(ctx.seat)];
                    let title = if score_ctx.title.is_empty() {
                        format!("{}符{}飜", score_ctx.fu, score_ctx.fan)
                    } else {
                        if score_ctx.yakuman != 0 || score_ctx.fan >= 13 {
                            "役満"
                        } else {
                            &score_ctx.title
                        }
                        .to_string()
                    };
                    if ctx.is_drawn {
                        if score_ctx.points.2 == 0 {
                            detail.push(json!(format!("{}{}点∀", title, score_ctx.points.1)));
                        } else {
                            detail.push(json!(format!(
                                "{}{}-{}点",
                                title, score_ctx.points.1, score_ctx.points.2,
                            )));
                        }
                    } else {
                        detail.push(json!(format!("{}{}点", title, score_ctx.points.0)));
                    }
                    for y in &score_ctx.yakus {
                        detail.push(json!(format!("{}({}飜)", y.name, y.fan)));
                    }
                    k.result_detail.push(detail);
                }
            }
            Event::Draw(ev) => {
                // TODO
                k.result = "流局".to_string();
                k.result_detail
                    .push(ev.delta_scores.iter().map(|&p| json!(p)).collect());
            }
            Event::End(_) => {}
        }
    }

    pub fn serialize(&mut self) -> String {
        self.log.log = vec![self.dealer.to_log()];
        serde_json::to_string(&self.log).unwrap()
    }
}

impl Default for TenhouSerializer {
    fn default() -> Self {
        Self::new()
    }
}

// [TenhouDeserializer]
// struct TenhouDeserializer {}

fn tile_to_tenhou(t: Tile) -> i64 {
    (match t {
        Z8 => 0,                            // Unknown
        Tile(ti, 0) => 50 + ti + 1,         // 赤ドラ
        Tile(ti, ni) => (ti + 1) * 10 + ni, // 通常
    }) as i64
}

// fn tile_from_tenhou(t: i64) -> Tile {
//     let t = t as usize;
//     match t {
//         0 => Z8,
//         11..=47 => Tile(t / 10 - 1, t % 10),
//         51..=53 => Tile(t % 10 - 1, 0),
//         _ => panic!("invalid tile number: {}", t),
//     }
// }

fn tiles_to_tenhou(v: &[Tile]) -> Vec<i64> {
    v.iter().map(|&t| tile_to_tenhou(t)).collect()
}

// fn tiles_from_tenhou(v: &[i64]) -> Vec<Tile> {
//     v.iter().map(|&t| tile_from_tenhou(t)).collect()
// }
