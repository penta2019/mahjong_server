use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::{error, info};

pub type SendRecv = Arc<Mutex<Option<(mpsc::Sender<String>, mpsc::Receiver<String>)>>>;

pub fn create_ws_server(port: u32) -> SendRecv {
    let sr = Arc::new(Mutex::new(None));
    let sr1 = sr.clone();
    thread::spawn(move || {
        ws::listen(format!("127.0.0.1:{}", port), |out| {
            let mut sr2 = sr1.lock().unwrap();
            if let None = sr2.as_ref() {
                let (su, ru) = mpsc::channel();
                let (sd, rd) = mpsc::channel();

                thread::spawn(move || loop {
                    if let Ok(s) = rd.recv() {
                        out.send(s).ok();
                    } else {
                        break;
                    }
                });
                *sr2 = Some((sd, ru));

                WsHandler {
                    port: port,
                    send_up: Some(su),
                    send_recv: sr1.clone(),
                }
            } else {
                // ignore duplicated connection
                WsHandler {
                    port: port,
                    send_up: None,
                    send_recv: sr1.clone(),
                }
            }
        })
        .unwrap();
    });

    sr
}

struct WsHandler {
    port: u32,
    send_up: Option<mpsc::Sender<String>>,
    send_recv: SendRecv,
}

impl ws::Handler for WsHandler {
    fn on_open(&mut self, _: ws::Handshake) -> ws::Result<()> {
        if let Some(_) = &self.send_up {
            info!("open connection (port: {})", self.port);
        } else {
            error!("invalid connection (port: {})", self.port);
        }
        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        if let Some(s) = &self.send_up {
            s.send(msg.to_string()).unwrap();
        }
        Ok(())
    }

    fn on_close(&mut self, _: ws::CloseCode, _: &str) {
        if let Some(_) = &self.send_up {
            self.send_up = None;
            *self.send_recv.lock().unwrap() = None;
            info!("close connection (port: {})", self.port);
        } else {
            info!("close invalid connection (port: {})", self.port);
        }
    }
}
