use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    // clippyの警告が出るのでBox化　#large_enum_variant
    Event(Box<Event>),
    Action {
        actions: Vec<Action>,
        tenpais: Vec<Tenpai>,
    },
    Info {
        seat: Seat,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Action(Action),
}
