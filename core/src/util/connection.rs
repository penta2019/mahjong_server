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
    Close,
    NoMessage,
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
        let conn = Self {
            stream: None,
            rx: rx,
        };

        let listener = TcpListener::bind(&addr).unwrap();
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
            stream.write((msg.to_string() + "\n").as_bytes()).ok();
        }
    }

    fn recv(&mut self) -> Message {
        if let Ok(stream) = self.rx.try_recv() {
            if let None = self.stream {
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

        if let None = self.stream {
            return Message::NoConnection;
        }

        let stream = self.stream.as_mut().unwrap();
        let mut reader = std::io::BufReader::new(stream);
        let mut buf = String::new();
        match reader.read_line(&mut buf) {
            Ok(size) => {
                if size != 0 {
                    if buf.chars().last() == Some('\n') {
                        buf.pop();
                    }
                    return Message::Text(buf);
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    return Message::NoMessage;
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

// TCP
// メッセージを'\n'で区切るのでメッセージ自体に'\n'は含めることはできない
type WsStream = websocket::sync::Client<TcpStream>;
pub struct WsConnection {
    stream: Option<WsStream>,
    rx: mpsc::Receiver<WsStream>,
}

impl WsConnection {
    pub fn new(addr: &str) -> Self {
        let (tx, rx) = mpsc::channel();
        let conn = Self {
            stream: None,
            rx: rx,
        };

        let listener = websocket::sync::Server::bind(addr).unwrap();
        thread::spawn(move || {
            for request in listener.filter_map(Result::ok) {
                tx.send(request.accept().unwrap()).unwrap();
            }
        });

        conn
    }
}

impl Connection for WsConnection {
    fn send(&mut self, msg: &str) {
        if let Some(stream) = self.stream.as_mut() {
            stream
                .send_message(&websocket::OwnedMessage::Text(msg.to_string()))
                .ok();
        }
    }

    fn recv(&mut self) -> Message {
        if let Ok(stream) = self.rx.try_recv() {
            if let None = self.stream {
                stream.set_nonblocking(true).unwrap();
                info!("ws connection opened from: {}", stream.peer_addr().unwrap());
                self.stream = Some(stream);

                return Message::Open;
            } else {
                error!("ws duplicated connection");
            }
        }

        if let None = self.stream {
            return Message::NoConnection;
        }

        let stream = self.stream.as_mut().unwrap();
        loop {
            use websocket::OwnedMessage;
            match stream.recv_message() {
                Ok(msg) => match msg {
                    OwnedMessage::Close(_) => {
                        let msg = OwnedMessage::Close(None);
                        stream.send_message(&msg).ok();
                        break;
                    }
                    OwnedMessage::Ping(ping) => {
                        let msg = OwnedMessage::Pong(ping);
                        stream.send_message(&msg).ok();
                    }
                    OwnedMessage::Text(text) => {
                        return Message::Text(text);
                    }
                    _ => {
                        warn!("ws unhandled message: {:?}", msg);
                    }
                },
                Err(e) => {
                    use websocket::WebSocketError;
                    match e {
                        WebSocketError::NoDataAvailable => return Message::NoMessage,
                        WebSocketError::IoError(_) => return Message::NoMessage, // for set_nonblocking
                        _ => error!("{}", e),
                    }
                    break;
                }
            }
        }

        self.stream = None;
        info!("ws connection closed");
        Message::Close
    }
}

#[test]
fn conn() {
    use crate::util::common::*;
    let mut conn = WsConnection::new("127.0.0.1:12345");
    loop {
        // println!("loop");
        let msg = conn.recv();
        println!("{:?}", msg);
        if let Message::Text(t) = msg {
            conn.send(&t);
        }
        sleep_ms(100);
    }
    // prompt();
}
