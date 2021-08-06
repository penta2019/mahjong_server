use serde_json::json;

use crate::controller::Listener;
use crate::model::*;
use crate::util::server::Server;

pub struct GuiServer {
    server: Server,
}

impl GuiServer {
    pub fn new(port: u32) -> Self {
        Self {
            server: Server::new_ws_server(&format!("localhost:{}", port)),
        }
    }
}

impl Listener for GuiServer {
    fn notify_event(&mut self, stg: &Stage, _event: &Event) {
        if self.server.is_connected() {
            let value = json!({
                "type": "stage",
                "data": stg,
            });
            self.server.send(value.to_string());
        }
    }
}
