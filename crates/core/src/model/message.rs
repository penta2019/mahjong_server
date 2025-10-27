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
