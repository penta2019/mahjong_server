use std::{
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender, channel},
    },
    task::{Context, Poll, Waker},
    thread,
};

pub use super::*;
use crate::model::{MessageHolder, SelectedAction};

pub struct GuiBuilder;

impl ActorBuilder for GuiBuilder {
    fn get_default_config(&self) -> Config {
        Config {
            name: "Gui".to_string(),
            args: vec![Arg::bool("conceal", true)],
        }
    }

    fn create(&self, config: Config) -> Box<dyn Actor> {
        Box::new(Gui::from_config(config))
    }
}

#[derive(Debug)]
struct Shared {
    possible_actions: Option<PossibleActions>,
    selected_action: Option<SelectedAction>,
    waker: Option<Waker>,
}

#[derive(Debug)]
pub struct Gui {
    config: Config,
    conceal: bool,
    messages: MessageHolder,
    tx: Sender<ServerMessage>,
    shared: Arc<Mutex<Shared>>,
    client_txrx: Option<(Sender<ClientMessage>, Receiver<ServerMessage>)>,
}

impl Gui {
    pub fn from_config(config: Config) -> Self {
        let (tx, client_rx) = channel();
        let (client_tx, rx) = channel();
        let conceal = config.args[0].value.as_bool();

        let shared = Arc::new(Mutex::new(Shared {
            possible_actions: None,
            selected_action: None,
            waker: None,
        }));
        let shared2 = shared.clone();

        thread::spawn(move || {
            while let Ok(act) = rx.recv() {
                let mut shared = shared2.lock().unwrap();
                match act {
                    ClientMessage::Action(act) => {
                        if let Some(actions) = &shared.possible_actions
                            && act.id == actions.id
                        {
                            // TODO: actがactionsに含まれるかチェック
                            shared.selected_action = Some(act);
                            if let Some(waker) = shared.waker.take() {
                                waker.wake();
                            }
                        }
                    }
                }
            }
        });

        Self {
            config,
            conceal,
            messages: MessageHolder::new(NO_SEAT, conceal),
            tx,
            shared,
            client_txrx: Some((client_tx, client_rx)),
        }
    }

    pub fn take_client_txrx(&mut self) -> Option<(Sender<ClientMessage>, Receiver<ServerMessage>)> {
        self.client_txrx.take()
    }

    fn flush(&mut self) {
        let mut shared = self.shared.lock().unwrap();
        while let Some(msg) = self.messages.next() {
            shared.possible_actions = None;
            shared.selected_action = None;
            if let ServerMessage::Action(possible_actions) = msg {
                shared.possible_actions = Some(possible_actions.clone());
            }
            self.tx.send(msg.clone()).unwrap();
        }
    }
}

impl Clone for Gui {
    fn clone(&self) -> Self {
        panic!("Actor 'Gui' can't be cloned");
    }
}

impl Actor for Gui {
    fn init(&mut self, _stage: StageRef, seat: Seat) {
        self.messages = MessageHolder::new(seat, self.conceal);
    }

    fn select(&mut self, acts: &[Action], tenpais: &[Tenpai]) -> ActionFuture {
        self.messages.push_actions(acts.to_vec(), tenpais.to_vec());
        self.flush();
        Box::pin(SelectFuture {
            shared: self.shared.clone(),
        })
    }

    fn expire(&mut self) {
        self.shared.lock().unwrap().possible_actions = None;
    }

    fn get_config(&self) -> &Config {
        &self.config
    }

    fn try_as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }
}

impl Listener for Gui {
    fn notify_event(&mut self, _stg: &Stage, event: &Event) {
        self.messages.push_event(event.clone());
        self.flush();
    }
}

struct SelectFuture {
    shared: Arc<Mutex<Shared>>,
}

impl Future for SelectFuture {
    type Output = Action;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared = self.shared.lock().unwrap();
        if shared.possible_actions.is_none() {
            return Poll::Ready(Action::nop());
        }
        if shared.selected_action.is_none() {
            shared.waker = Some(cx.waker().clone());
            return Poll::Pending;
        }
        Poll::Ready(shared.selected_action.take().unwrap().action)
    }
}
