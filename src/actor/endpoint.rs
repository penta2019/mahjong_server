use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;

use serde_json::{json, Value};

use super::*;
use crate::util::connection::{Connection, Message, TcpConnection};
use crate::util::misc::sleep;

use crate::error;

pub struct EndpointBuilder;

#[derive(Debug, Default)]
struct SharedData {
    msgs: Vec<(Value, bool)>, // [(message, is_action)]
    cursor: usize,
    action: Option<Action>,
    waker: Option<Waker>,
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
    shared: Arc<Mutex<SharedData>>,
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
            sleep(0.01); // 負荷軽減&Lock解除時間
            let mut d = arc1.lock().unwrap();
            match conn.recv() {
                Message::Open => d.cursor = 0,
                Message::Text(act) => match serde_json::from_str::<Action>(&act) {
                    Ok(a) => {
                        d.action = Some(a);
                        d.waker.take().unwrap().wake();
                    }
                    Err(e) => error!("{}: {}", e, act),
                },
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
                }
                Message::Close => {}
                Message::NoConnection => {}
            }
        });

        Self {
            config,
            shared: arc0,
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
        *self.shared.lock().unwrap() = SharedData::default();
        self.seat = seat;
    }

    fn select_action(&mut self, acts: &[Action], tenpais: &[Tenpai]) -> SelectedAction {
        let mut shared = self.shared.lock().unwrap();
        let act_msg = json!({"type": "Action", "actions": json!(acts), "tenpais": json!(tenpais)});
        shared.msgs.push((act_msg, true));
        shared.action = None;

        Box::pin(SelectFuture {
            shared: self.shared.clone(),
        })
    }

    fn get_config(&self) -> &Config {
        &self.config
    }
}

impl Listener for Endpoint {
    fn notify_event(&mut self, _stg: &Stage, event: &Event) {
        let mut d = self.shared.lock().unwrap();
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
    }
}

pub struct SelectFuture {
    shared: Arc<Mutex<SharedData>>,
}

impl Future for SelectFuture {
    type Output = Action;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared = self.shared.lock().unwrap();
        if shared.action.is_none() {
            shared.waker = Some(cx.waker().clone());
            return Poll::Pending;
        }

        Poll::Ready(shared.action.take().unwrap())
    }
}
