use mahjong_core::{
    control::{common::*, string::*},
    error,
    util::misc::prompt,
};

use super::*;

use ActionType::*;

pub struct ManualBuilder;

impl ActorBuilder for ManualBuilder {
    fn get_default_config(&self) -> Config {
        Config {
            name: "Manual".into(),
            args: vec![],
        }
    }

    fn create(&self, config: Config) -> Box<dyn Actor> {
        Box::new(Manual::from_config(config))
    }
}

#[derive(Clone)]
pub struct Manual {
    config: Config,
    stage: StageRef,
    seat: Seat,
}

impl Manual {
    pub fn from_config(config: Config) -> Self {
        Self {
            config,
            stage: StageRef::default(),
            seat: NO_SEAT,
        }
    }
}

impl Actor for Manual {
    fn init(&mut self, stage: StageRef, seat: Seat) {
        self.stage = stage;
        self.seat = seat;
    }

    // この関数は標準入力でブロッキングする.
    // これは複数のManual Actorを使用する場合に入力を区別するための仕様である.
    // テスト用のActorなので許容するが,この挙動は本来は望ましくない
    fn select(&mut self, acts: &[Action], _tenpais: &[Tenpai]) -> ActionFuture {
        let stg = self.stage.lock().unwrap();
        let pl = &stg.players[self.seat];
        let mut hand = pl.hand;
        if let Some(t) = pl.drawn {
            dec_tile(&mut hand, t);
        }
        let mut hand_str = tiles_to_string(&tiles_from_tile_table(&hand));
        if stg.turn == self.seat {
            println!("[Turn Action] select action or discard tile");
            if let Some(t) = pl.drawn {
                hand_str.push_str(&format!(" {}", &t.to_string()));
            }
        } else {
            println!("[Call Action] select action");
            hand_str.push_str(&format!(" ({})", stg.last_tile.unwrap().2));
        }
        for m in &pl.melds {
            hand_str.push_str(&format!(",{}", &meld_to_string(m, pl.seat)));
        }
        println!("{}", hand_str);

        for (i, act) in acts.iter().enumerate() {
            let i = if act.ty == Discard {
                "_".into()
            } else {
                i.to_string()
            };
            if i == "0" {
                println!("0(default) => {}", act);
            } else {
                println!("{} => {}", i, act);
            }
        }

        let mut riichi = false;
        loop {
            if riichi {
                print!("riichi ");
            }
            let buf = prompt();
            let mut chars = buf.chars();
            let ch = if let Some(ch) = chars.next() {
                ch
            } else {
                println!();
                continue;
            };
            match ch {
                'm' | 'p' | 's' | 'z' => {
                    if stg.turn != self.seat {
                        error!("discard not allowed");
                        continue;
                    }

                    let ti = tile_type_from_char(ch).unwrap();
                    let ni = match tile_number_from_char(chars.next().unwrap()) {
                        Ok(n) => n,
                        Err(_) => {
                            error!("invalid tile symbol");
                            continue;
                        }
                    };

                    let h = &stg.players[self.seat].hand;
                    let t = Tile(ti, ni);
                    if t.0 > TZ || t.1 > 9 {
                        error!("invalid tile: {}", t);
                        continue;
                    } else if h[t.0][t.1] == 0 {
                        error!("tile not found: {}", t);
                        continue;
                    }

                    if riichi {
                        if let Some(act) = acts.iter().find(|a| a.ty == Riichi) {
                            if act.tiles.contains(&t) {
                                println!();
                                return ready(Action::riichi(t));
                            } else {
                                error!("invalid Riichi tile");
                            }
                        } else {
                            panic!();
                        }
                    } else {
                        if let Some(act) = acts.iter().find(|a| a.ty == Discard) {
                            if !act.tiles.contains(&t) {
                                println!();
                                return ready(Action::discard(t));
                            } else {
                                error!("restricted tile after Chi or Pon");
                            }
                        } else {
                            error!("Discard is not allowed");
                        }
                    }
                }
                '!' => {
                    match &buf[1..] {
                        "print\n" => {
                            println!("{}", stg);
                        }
                        _ => {
                            error!("unknown command: {}", &buf[1..]);
                        }
                    }
                    continue;
                }
                '\n' => {
                    return ready(acts[0].clone());
                    // continue;
                }
                _ => {
                    let n: usize = match buf.trim().parse() {
                        Ok(n) => n,
                        Err(_) => {
                            error!("input must be a number or tile symbol");
                            continue;
                        }
                    };
                    if n >= acts.len() {
                        error!("invalid action index");
                        continue;
                    }

                    match acts[n].ty {
                        Discard => {
                            println!("please select tile");
                        }
                        Riichi => {
                            riichi = true;
                        }
                        _ => {
                            println!();
                            return ready(acts[n].clone());
                        }
                    }
                }
            };
        }
    }

    fn get_config(&self) -> &Config {
        &self.config
    }
}

impl Listener for Manual {}
