use std::sync::{Arc, Mutex};
use std::thread;

use serde_json::{json, Value};

use super::*;
use crate::util::common::sleep_ms;
use crate::util::connection::{Connection, Message, TcpConnection};

use crate::error;

pub struct EndpointBuilder;

#[derive(Debug, Default)]
struct SharedData {
    send_request: bool,
    msgs: Vec<Value>,
    cursor: usize,
    action: Option<Action>,
}

impl ActorBuilder for EndpointBuilder {
    fn get_default_config(&self) -> Config {
        Config {
            name: "Endpoint".to_string(),
            args: vec![Arg::string("addr", "127.0.0.1:52010")],
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
    is_selecting: bool,
}

impl Endpoint {
    pub fn from_config(config: Config) -> Self {
        let args = &config.args;
        let addr = args[0].value.as_string();
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
                        // println!("{:?}", act);
                        match serde_json::from_str::<Action>(&act) {
                            Ok(a) => d.action = Some(a),
                            Err(e) => error!("{}: {}", e, act),
                        }
                    }
                    Message::NoMessage => {
                        while d.cursor < d.msgs.len() {
                            conn.send(&d.msgs[d.cursor].to_string());
                            d.cursor += 1;
                        }
                        break;
                    }
                    Message::Close => {}
                    Message::NoConnection => break,
                }
            }
            sleep_ms(10);
        });

        Self {
            config,
            data: arc0,
            seat: NO_SEAT,
            is_selecting: false,
        }
    }
}

impl Clone for Endpoint {
    fn clone(&self) -> Self {
        panic!("Actor 'Endpoint' can't be cloned");
    }
}

impl Actor for Endpoint {
    fn init(&mut self, seat: Seat) {
        *self.data.lock().unwrap() = SharedData::default();
        self.seat = seat;
        self.is_selecting = false;
    }

    fn select_action(&mut self, _stg: &Stage, acts: &Vec<Action>, repeat: i32) -> Option<Action> {
        let mut ret = None;
        let mut d = self.data.lock().unwrap();
        if repeat == 0 {
            let act_msg = json!({"type": "Action", "actions": json!(acts)});
            d.msgs.push(act_msg);
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
            let mut val = json!(event);
            match event {
                Event::New(_) => {
                    val["seat"] = json!(self.seat);
                }
                _ => {}
            }
            d.msgs.push(val);
            d.send_request = true;
        }

        while self.data.lock().unwrap().send_request {
            sleep_ms(10); // pushしたデータが処理されるまで待機
        }
    }
}
