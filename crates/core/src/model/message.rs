use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    // clippyの警告が出るのでBox化 #large_enum_variant
    Event(Box<Event>),
    Action(PossibleActions),
    Info { seat: Seat },
    Log, // TODO
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Action(SelectedAction),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PossibleActions {
    pub id: u32, // 任意のid
    pub actions: Vec<Action>,
    pub tenpais: Vec<Tenpai>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedAction {
    pub id: u32, // ServerMessageの対応するActionのidをそのままコピー (誤動作防止用)
    pub action: Action,
}

#[derive(Debug)]
pub struct MessageHolder {
    seat: Seat,
    conceal: bool,
    messages: Vec<ServerMessage>,
    cursor: usize,
    act_id: u32,
}

impl MessageHolder {
    pub fn new(seat: Seat, conceal: bool) -> Self {
        Self {
            seat,
            conceal,
            messages: vec![],
            cursor: 0,
            act_id: 0,
        }
    }

    pub fn push_event(&mut self, mut event: Event) {
        match &mut event {
            Event::New(ev) => {
                self.messages = vec![ServerMessage::Info { seat: self.seat }];
                if self.conceal {
                    for s in 0..SEAT {
                        if s != self.seat {
                            ev.hands[s].fill(Z8);
                        }
                    }
                    ev.wall = vec![];
                    ev.dora_wall = vec![];
                    ev.ura_dora_wall = vec![];
                    ev.replacement_wall = vec![];
                }
            }
            Event::Deal(ev) => {
                if self.conceal && self.seat != ev.seat {
                    ev.tile = Z8;
                }
            }
            _ => {}
        };
        self.messages.push(ServerMessage::Event(Box::new(event)));
    }

    pub fn push_actions(&mut self, actions: Vec<Action>, tenpais: Vec<Tenpai>) {
        self.act_id += 1;
        self.messages.push(ServerMessage::Action(PossibleActions {
            id: self.act_id,
            actions,
            tenpais,
        }));
    }

    // pub fn reset(&mut self) {
    //     self.cursor = 0;
    // }

    pub fn next_message(&mut self) -> Option<&ServerMessage> {
        'skip: while self.cursor < self.messages.len() {
            let cursor = self.cursor;
            self.cursor += 1;

            // Actionよりも後ろにEventが存在する場合,失効済みなのでスキップ
            if let ServerMessage::Action { .. } = &self.messages[cursor] {
                for m in &self.messages[self.cursor..] {
                    if let ServerMessage::Event(_) = m {
                        continue 'skip;
                    }
                }
            }

            return Some(&self.messages[cursor]);
        }

        None
    }
}
