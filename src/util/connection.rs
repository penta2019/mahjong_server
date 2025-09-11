use std::fmt;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;

use crate::{error, info, warn};

#[derive(Debug)]
pub enum Message {
    Open,
    Text(String),
    Nop,
    Close,
    NoConnection,
}

pub trait Connection: Send {
    fn send(&mut self, _msg: &str);
    fn recv(&mut self) -> Message;
}

impl fmt::Debug for dyn Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "dyn Connection")
    }
}

// TCP
// メッセージを'\n'で区切るのでメッセージ自体に'\n'は含めることはできない
pub struct TcpConnection {
    stream: Option<TcpStream>,
    rx: mpsc::Receiver<TcpStream>,
}

impl TcpConnection {
    pub fn new(addr: &str) -> Self {
        let (tx, rx) = mpsc::channel();
        let conn = Self { stream: None, rx };

        let listener = TcpListener::bind(addr).unwrap();
        thread::spawn(move || {
            for request in listener.incoming() {
                match request {
                    Ok(stream) => tx.send(stream).unwrap(),
                    Err(e) => error!("tcp error: {}", e),
                }
            }
        });

        conn
    }
}

impl Connection for TcpConnection {
    fn send(&mut self, msg: &str) {
        if let Some(stream) = self.stream.as_mut() {
            stream.write_all((msg.to_string() + "\n").as_bytes()).ok();
        }
    }

    fn recv(&mut self) -> Message {
        if let Ok(stream) = self.rx.try_recv() {
            if self.stream.is_none() {
                stream.set_nonblocking(true).unwrap();
                info!(
                    "tcp connection opened from: {}",
                    stream.peer_addr().unwrap()
                );
                self.stream = Some(stream);

                return Message::Open;
            } else {
                error!("tcp duplicated connection");
            }
        }

        if self.stream.is_none() {
            return Message::NoConnection;
        }

        let stream = self.stream.as_mut().unwrap();
        let mut reader = std::io::BufReader::new(stream);
        let mut buf = String::new();
        match reader.read_line(&mut buf) {
            Ok(size) => {
                if size != 0 {
                    if buf.ends_with('\n') {
                        buf.pop();
                    }
                    return Message::Text(buf);
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    return Message::Nop;
                } else {
                    error!("{}", e);
                }
            }
        }

        self.stream = None;
        info!("tcp connection closed");
        Message::Close
    }
}

// websocket
pub struct WsConnection {
    stream: Option<tungstenite::protocol::WebSocket<TcpStream>>,
    rx: mpsc::Receiver<TcpStream>,
}

impl WsConnection {
    pub fn new(addr: &str) -> Self {
        let (tx, rx) = mpsc::channel();
        let conn = Self { stream: None, rx };

        let listener = TcpListener::bind(addr).unwrap();
        thread::spawn(move || {
            for request in listener.incoming() {
                match request {
                    Ok(stream) => tx.send(stream).unwrap(),
                    Err(e) => error!("ws error: {}", e),
                }
            }
        });

        conn
    }
}

impl Connection for WsConnection {
    fn send(&mut self, msg: &str) {
        if let Some(stream) = self.stream.as_mut() {
            stream.send(msg.into()).ok();
        }
    }

    fn recv(&mut self) -> Message {
        if let Ok(stream) = self.rx.try_recv() {
            if self.stream.is_none() {
                stream.set_nonblocking(true).unwrap();

                info!("ws connection opened from: {}", stream.peer_addr().unwrap());
                match tungstenite::accept(stream) {
                    Ok(s) => self.stream = Some(s),
                    Err(e) => error!("ws upgrade error: {}", e),
                }

                return Message::Open;
            } else {
                error!("ws duplicated connection");
            }
        }

        if self.stream.is_none() {
            return Message::NoConnection;
        }

        let stream = self.stream.as_mut().unwrap();
        loop {
            use tungstenite::protocol::Message as WsMessage;
            match stream.read() {
                Ok(msg) => match msg {
                    WsMessage::Close(_) => {
                        stream.send(WsMessage::Close(None)).ok();
                        break;
                    }
                    WsMessage::Ping(ping) => {
                        stream.send(WsMessage::Pong(ping)).ok();
                    }
                    WsMessage::Text(text) => {
                        return Message::Text(
                            String::from_utf8(text.as_bytes().to_owned()).unwrap(),
                        );
                    }
                    _ => {
                        warn!("ws unhandled message: {:?}", msg);
                    }
                },
                Err(e) => {
                    use tungstenite::error::Error as WsError;
                    if let WsError::Io(e) = &e
                        && e.kind() == std::io::ErrorKind::WouldBlock
                    {
                        return Message::Nop;
                    }

                    error!("ws error: {:?}", e);
                    break;
                }
            }
        }

        self.stream = None;
        info!("ws connection closed");
        Message::Close
    }
}

// #[test]
// fn conn() {
//     use crate::etc::misc::*;
//     let mut conn = WsConnection::new("127.0.0.1:12345");
//     loop {
//         let msg = conn.recv();
//         println!("{:?}", msg);
//         if let Message::Text(t) = msg {
//             conn.send(&t);
//         }
//         sleep(0.1);
//     }
// }
