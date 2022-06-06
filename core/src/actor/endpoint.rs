use serde_json::Value;

use super::*;
use crate::util::connection::{Connection, Message, WsConnection};

pub struct EndpointBuilder;

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
    conn: Box<dyn Connection>,
    is_conn: bool,
    record: Vec<Value>,
    cursor: usize,
    is_selecting: bool,
}

impl Endpoint {
    pub fn from_config(config: Config) -> Self {
        let args = &config.args;
        let addr = args[0].value.as_string();
        let conn = Box::new(WsConnection::new(&addr));
        Self {
            config: config,
            conn: conn,
            is_conn: false,
            record: vec![],
            cursor: 0,
            is_selecting: false,
        }
    }

    fn handle_conn(&mut self) -> Option<Action> {
        let mut res = None;
        loop {
            match self.conn.recv() {
                Message::Open => {
                    self.is_conn = true;
                    self.cursor = 0;
                }
                Message::Text(t) => {
                    let a = serde_json::from_str::<Action>(&t);
                    match a {
                        Ok(act) => res = Some(act),
                        Err(e) => error!("{}", e),
                    }
                }
                Message::Close => {
                    self.is_conn = false;
                    self.is_selecting = false;
                }
                _ => return res,
            }
        }
    }

    fn flush_record(&mut self) {
        if self.is_conn {
            while self.cursor < self.record.len() {
                self.conn.send(&self.record[self.cursor].to_string());
                self.cursor += 1;
                self.is_selecting = false;
            }
        }
    }
}

impl Clone for Endpoint {
    fn clone(&self) -> Self {
        panic!("Actor 'Endpoint' can't be cloned");
    }
}

impl Actor for Endpoint {
    fn init(&mut self, _seat: Seat) {
        self.record.clear();
        self.cursor = 0;
    }

    fn select_action(&mut self, _stg: &Stage, acts: &Vec<Action>, _repeat: i32) -> Option<Action> {
        println!("{}", serde_json::to_value(acts).unwrap().to_string());
        self.handle_conn();
        self.flush_record(); // select_action中に再接続した場合
        if self.is_conn && !self.is_selecting {
            self.conn
                .send(&serde_json::to_value(acts).unwrap().to_string());
            self.is_selecting = true;
        }

        self.handle_conn()
    }

    fn get_config(&self) -> &Config {
        &self.config
    }
}

impl Listener for Endpoint {
    fn notify_event(&mut self, stg: &Stage, event: &Event) {
        match event {
            Event::New(e) => {}
            Event::Deal(e) => {}
            _ => {}
        }
        self.record.push(serde_json::to_value(event).unwrap());

        self.handle_conn();
        self.flush_record();
    }
}
