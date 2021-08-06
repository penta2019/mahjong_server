use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::{error, info, warn};

// tcp, websocket等のサーバーを単一のテキスト用双方向コネクションに抽象化するモジュール
// * 通信の同期は行わない
// * 同時接続クライアントは1つのみ
// * 接続していない状態で送信されたメッセージはすべて破棄
// * 接続状態は把握できない
pub struct Server {
    sender: mpsc::Sender<String>,
    reciever: mpsc::Receiver<String>,
    event: mpsc::Receiver<ServerEvent>,
    is_new: bool,
    is_connected: bool,
}

impl Server {
    pub fn new_ws_server(addr: &str) -> Self {
        create_ws_server(addr)
    }

    pub fn new_tcp_server(addr: &str) -> Self {
        create_tcp_server(addr)
    }

    pub fn is_new(&mut self) -> bool {
        while let Ok(e) = self.event.try_recv() {
            self.update_flags(e);
        }
        let is_new = self.is_new;
        self.is_new = false;
        is_new
    }

    pub fn is_connected(&mut self) -> bool {
        while let Ok(e) = self.event.try_recv() {
            self.update_flags(e);
        }
        self.is_connected
    }

    pub fn wait_connected(&mut self) {
        loop {
            while let Ok(e) = self.event.try_recv() {
                self.update_flags(e);
            }
            if self.is_connected {
                return;
            }
            self.update_flags(self.event.recv().unwrap());
        }
    }

    pub fn recv(&mut self) -> String {
        self.reciever.recv().unwrap()
    }

    pub fn recv_try(&mut self) -> Option<String> {
        self.reciever.try_recv().ok()
    }

    // バグでクラッシュする https://github.com/rust-lang/rust/issues/39364
    // pub fn recv_timeout(&mut self, millis: u64) -> Option<String> {
    //     let d = std::time::Duration::from_millis(millis);
    //     self.reciever.recv_timeout(d).ok()
    // }
    //
    // Minimal Reproducer
    //     fn main() {
    //         use std::thread;
    //         use std::time::Duration;
    //         let (s, r) = std::sync::mpsc::channel::<String>();
    //         thread::spawn(move || {
    //             let _ = s.clone();
    //             thread::sleep(Duration::from_millis(1000));
    //         });
    //         r.recv_timeout(Duration::from_millis(1)).ok();
    //         r.recv().ok();
    //      }

    pub fn recv_timeout(&mut self, millis: u64) -> Option<String> {
        let mut ellapsed = 0;
        loop {
            match self.reciever.try_recv() {
                Ok(m) => return Some(m),
                Err(_) => {
                    if ellapsed > millis {
                        return None;
                    }
                    crate::util::common::sleep_ms(100);
                    ellapsed += 100;
                }
            }
        }
    }

    pub fn send(&mut self, message: String) {
        self.sender.send(message).ok();
    }

    fn update_flags(&mut self, event: ServerEvent) {
        match event {
            ServerEvent::Open => {
                self.is_new = true;
                self.is_connected = true;
            }
            ServerEvent::Close => {
                self.is_new = false;
                self.is_connected = false;
            }
        }
    }
}

enum ServerEvent {
    Open,
    Close,
}

fn create_ws_server(addr: &str) -> Server {
    use websocket::OwnedMessage;

    fn ws_send(
        sender: &Arc<Mutex<Option<websocket::sender::Writer<std::net::TcpStream>>>>,
        message: &OwnedMessage,
    ) {
        match sender.lock().unwrap().as_mut() {
            Some(s) => s.send_message(message).unwrap(),
            None => error!("ws failed to send message: no connection"),
        }
    }

    let (su, ru) = mpsc::channel(); // upstream
    let (sd, rd) = mpsc::channel(); // downstream
    let (se, re) = mpsc::channel(); // event

    let server = websocket::sync::Server::bind(addr).unwrap();
    let sender = Arc::new(Mutex::new(None));
    info!("ws listening on {}", addr);

    let sender2 = sender.clone();
    thread::spawn(move || loop {
        ws_send(&sender2, &OwnedMessage::Text(rd.recv().unwrap()));
    });

    thread::spawn(move || {
        for request in server.filter_map(Result::ok) {
            let client = request.accept().unwrap();
            match *sender.lock().unwrap() {
                Some(_) => {
                    error!("ws duplicated connection");
                    continue;
                }
                None => {}
            }

            let sender = sender.clone();
            let su = su.clone();
            let se = se.clone();
            thread::spawn(move || {
                info!("ws connection from: {}", client.peer_addr().unwrap());
                let (mut r, s) = client.split().unwrap();
                *sender.lock().unwrap() = Some(s);
                se.send(ServerEvent::Open).ok();

                for message in r.incoming_messages() {
                    match message {
                        Ok(om) => match om {
                            OwnedMessage::Close(_) => {
                                let message = OwnedMessage::Close(None);
                                ws_send(&sender, &message);
                            }
                            OwnedMessage::Ping(ping) => {
                                let message = OwnedMessage::Pong(ping);
                                ws_send(&sender, &message);
                            }
                            OwnedMessage::Text(text) => {
                                su.send(text).ok();
                            }
                            _ => {
                                warn!("ws unhandled message: {:?}", om);
                            }
                        },
                        Err(e) => {
                            error!("ws: {}", e);
                            break;
                        }
                    }
                }

                info!("ws connection closed");
                *sender.lock().unwrap() = None;
                se.send(ServerEvent::Close).ok();
            });
        }
    });

    Server {
        sender: sd,
        reciever: ru,
        event: re,
        is_new: false,
        is_connected: false,
    }
}

// メッセージを'\n'で区切るのでメッセージ自体に'\n'は含めることはできない
fn create_tcp_server(addr: &str) -> Server {
    use std::io::prelude::*;
    use std::io::BufReader;
    use std::net::{TcpListener, TcpStream};

    fn tcp_send(sender: &Arc<Mutex<Option<TcpStream>>>, message: &String) {
        match sender.lock().unwrap().as_mut() {
            Some(s) => {
                s.write(format!("{}\n", message).as_bytes()).ok();
            }
            None => {
                error!("ws failed to send message: no connection");
            }
        }
    }

    let (su, ru) = mpsc::channel(); // upstream
    let (sd, rd) = mpsc::channel(); // downstream
    let (se, re) = mpsc::channel(); // event

    let listener = TcpListener::bind(&addr).unwrap();
    let sender = Arc::new(Mutex::new(None));
    info!("tcp listening on {}", addr);

    let sender2 = sender.clone();
    thread::spawn(move || loop {
        tcp_send(&sender2, &rd.recv().unwrap());
    });

    thread::spawn(move || {
        for request in listener.incoming() {
            if let Err(e) = request {
                error!("tcp error: {}", e);
                continue;
            }

            match *sender.lock().unwrap() {
                Some(_) => {
                    error!("tcp duplicated connection");
                    continue;
                }
                None => {}
            }

            let sender = sender.clone();
            let su = su.clone();
            let se = se.clone();
            thread::spawn(move || {
                info!("tcp connection from: {:?}", request);
                let mut client = request.unwrap();
                let client2 = client.try_clone().unwrap();
                *sender.lock().unwrap() = Some(client2);
                se.send(ServerEvent::Open).ok();

                loop {
                    let mut buf_read = BufReader::new(&mut client);
                    let mut buf = String::new();
                    match buf_read.read_line(&mut buf) {
                        Ok(size) => {
                            if size == 0 {
                                break;
                            }
                            let text = buf[..buf.len() - 1].to_string(); // 行末の'\n'を削除
                            su.send(text).ok();
                        }
                        Err(e) => {
                            error!("tcp: {}", e);
                            break;
                        }
                    }
                }

                info!("tcp connection closed");
                *sender.lock().unwrap() = None;
                se.send(ServerEvent::Close).ok();
            });
        }
    });

    Server {
        sender: sd,
        reciever: ru,
        event: re,
        is_new: false,
        is_connected: false,
    }
}

fn run_server(mut srv: Server) {
    loop {
        if let Some(text) = srv.recv_timeout(1000) {
            println!("Text: {}", text);
            srv.send(text);
        } else {
            println!("Timeout");
        }
    }
    // loop {
    //     if let Some(text) = srv.recv_try() {
    //         println!("Text: {}", text);
    //         srv.send(text);
    //     }
    //     crate::util::common::sleep_ms(100);
    // }
}

#[allow(dead_code)]
fn run_ws_server() {
    let srv = Server::new_ws_server("127.0.0.1:12345");
    run_server(srv);
}

#[allow(dead_code)]
fn run_tcp_server() {
    let srv = Server::new_tcp_server("127.0.0.1:12345");
    run_server(srv);
}

#[test]
fn test_server() {
    run_tcp_server();
}
