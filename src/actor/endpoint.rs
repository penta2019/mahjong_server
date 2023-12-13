use std::sync::{Arc, Mutex};
use std::thread;

use serde_json::{json, Value};

use super::*;
use crate::util::connection::{Connection, Message, TcpConnection};
use crate::util::misc::sleep;

use crate::error;

pub struct EndpointBuilder;

#[derive(Debug, Default)]
struct SharedData {
    send_request: bool,
    msgs: Vec<(Value, bool)>, // [(message, is_action)]
    cursor: usize,
    action: Option<Action>,
}

impl ActorBuilder for EndpointBuilder {
    fn get_default_config(&self) -> Config {
        Config {
            name: "Endpoint".to_string(),
            args: vec![
                Arg::bool("debug", false),
                Arg::string("addr", "127.0.0.1:52010"),
            ],
        }
    }

    fn create(&self, config: Config) -> Box<dyn Actor> {
        Box::new(Endpoint::from_config(config))
    }
}

pub struct Endpoint {
    config: Config,
    data: Arc<Mutex<SharedData>>,
    seat: Seat,
    debug: bool,
}

impl Endpoint {
    pub fn from_config(config: Config) -> Self {
        let args = &config.args;
        let debug = args[0].value.as_bool();
        let addr = args[1].value.as_string();
        let mut conn = Box::new(TcpConnection::new(&addr));
        let arc0 = Arc::new(Mutex::new(SharedData::default()));
        let arc1 = arc0.clone();

        thread::spawn(move || loop {
            loop {
                let mut d = arc1.lock().unwrap();
                d.send_request = false;
                match conn.recv() {
                    Message::Open => d.cursor = 0,
                    Message::Text(act) => {
                        // println!("{}", act);
                        match serde_json::from_str::<Action>(&act) {
                            Ok(a) => d.action = Some(a),
                            Err(e) => error!("{}: {}", e, act),
                        }
                    }
                    Message::NoMessage => {
                        while d.cursor < d.msgs.len() {
                            let (msg, is_action) = &d.msgs[d.cursor];
                            if *is_action && d.cursor != d.msgs.len() - 1 {
                                // メッセージがアクションでかつ最後のメッセージでない場合は失効済みなので送信しない
                            } else {
                                conn.send(&msg.to_string());
                            }
                            d.cursor += 1;
                        }
                        break;
                    }
                    Message::Close => {}
                    Message::NoConnection => break,
                }
            }
            sleep(0.01);
        });

        Self {
            config,
            data: arc0,
            seat: NO_SEAT,
            debug,
        }
    }
}

impl Clone for Endpoint {
    fn clone(&self) -> Self {
        panic!("Actor 'Endpoint' can't be cloned");
    }
}

impl Actor for Endpoint {
    fn init(&mut self, _stage: StageRef, seat: Seat) {
        *self.data.lock().unwrap() = SharedData::default();
        self.seat = seat;
    }

    fn select_action(
        &mut self,
        _stg: &Stage,
        acts: &[Action],
        tenpais: &[Tenpai],
        repeat: i32,
    ) -> Option<Action> {
        let mut ret = None;
        let mut d = self.data.lock().unwrap();
        if repeat == 0 {
            let act_msg =
                json!({"type": "Action", "actions": json!(acts), "tenpais": json!(tenpais)});
            d.msgs.push((act_msg, true));
        } else {
            ret = d.action.clone();
        }
        d.action = None;

        ret
    }

    fn get_config(&self) -> &Config {
        &self.config
    }
}

impl Listener for Endpoint {
    fn notify_event(&mut self, _stg: &Stage, event: &Event) {
        {
            let mut d = self.data.lock().unwrap();
            let val = match event {
                Event::New(e) => {
                    let mut hands = e.hands.clone();
                    for s in 0..SEAT {
                        if !self.debug && s != self.seat {
                            hands[s].fill(Z8);
                        }
                    }
                    let e2 = Event::New(EventNew {
                        rule: e.rule.clone(),
                        hands,
                        doras: e.doras.clone(),
                        names: e.names.clone(),
                        ..*e
                    });
                    let mut val = json!(e2);
                    val["seat"] = json!(self.seat);
                    val
                }
                Event::Deal(e) => {
                    let t = if !self.debug && self.seat != e.seat {
                        Z8
                    } else {
                        e.tile
                    };
                    let e2 = Event::Deal(EventDeal { tile: t, ..*e });
                    json!(e2)
                }
                _ => json!(event),
            };
            d.msgs.push((val, false));
            d.send_request = true;
        }

        while self.data.lock().unwrap().send_request {
            sleep(0.01); // pushしたデータが処理されるまで待機
        }
    }
}
