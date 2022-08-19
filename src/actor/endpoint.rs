use std::sync::{Arc, Mutex};
use std::thread;

use serde_json::{json, Value};

use super::*;
use crate::util::common::sleep_ms;
use crate::util::connection::{Connection, Message, WsConnection};

pub struct EndpointBuilder;

#[derive(Debug, Default)]
struct SharedData {
    msgs: Vec<Value>,
    cursor: usize,
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
        let mut conn = Box::new(WsConnection::new(&addr));
        let arc0 = Arc::new(Mutex::new(SharedData::default()));
        let arc1 = arc0.clone();

        thread::spawn(move || loop {
            loop {
                let mut d = arc1.lock().unwrap();
                match conn.recv() {
                    Message::Open => d.cursor = 0,
                    Message::Text(_) => {}
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
            sleep_ms(100);
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

    fn select_action(&mut self, stg: &Stage, acts: &Vec<Action>, _repeat: i32) -> Option<Action> {
        // let act = self.handle_conn();
        // if self.is_selecting {
        //     return act;
        // }

        // self.flush_record(); // select_action中に再接続した場合
        // if self.is_conn && !self.is_selecting {
        //     println!("{}", &stg.players[self.seat]);
        //     self.conn
        //         .send(&serde_json::to_value(acts).unwrap().to_string());
        //     self.is_selecting = true;
        // }

        None
    }

    fn get_config(&self) -> &Config {
        &self.config
    }
}

impl Listener for Endpoint {
    fn notify_event(&mut self, _stg: &Stage, event: &Event) {
        let mut d = self.data.lock().unwrap();
        d.msgs.push(json!(event));
    }
}
