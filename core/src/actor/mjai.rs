use std::io;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use serde_json::{json, to_value, Value};

use super::*;
use crate::convert::mjai::*;
use crate::util::common::{flush, sleep_ms, vec_remove};

use crate::{error, info};

pub struct MjaiEndpointBuilder;

impl ActorBuilder for MjaiEndpointBuilder {
    fn get_default_config(&self) -> Config {
        Config {
            name: "MjaiEndpoint".to_string(),
            args: vec![
                Arg::string("addr", "127.0.0.1:11601"),
                Arg::int("timeout", 10),
            ],
        }
    }

    fn create(&self, config: Config) -> Box<dyn Actor> {
        Box::new(MjaiEndpoint::from_config(config))
    }
}

#[derive(Clone)]
pub struct MjaiEndpoint {
    config: Config,
    seat: usize,
    data: Arc<Mutex<SharedData>>,
    try_riichi: Option<Seat>,
    is_new_game: bool,
    timeout: i32, // selected_action の最大待機時間(秒)
    timeout_count: i32,
}

impl MjaiEndpoint {
    pub fn from_config(config: Config) -> Self {
        let args = &config.args;
        let addr = args[0].value.as_string();
        let timeout = args[1].value.as_int();

        let data = Arc::new(Mutex::new(SharedData::new()));
        let obj = Self {
            config: config,
            seat: NO_SEAT,
            data: data.clone(),
            try_riichi: None,
            is_new_game: false,
            timeout: timeout,
            timeout_count: 0,
        };

        let listener = TcpListener::bind(&addr).unwrap();
        info!("listening on {}", addr);

        thread::spawn(move || {
            let is_connected = Arc::new(Mutex::new(false));
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        if *is_connected.lock().unwrap() {
                            error!("duplicated connection");
                            continue;
                        }
                        *is_connected.lock().unwrap() = true;

                        let is_connected = is_connected.clone();
                        let data = data.clone();
                        thread::spawn(move || {
                            info!("new connection {:?}", stream);
                            match stream_handler(&mut stream, data, true) {
                                Ok(_) => {}
                                Err(e) => {
                                    error!("{:?}", e);
                                }
                            }
                            info!("connection closed");
                            *is_connected.lock().unwrap() = false;
                        });
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
        });

        obj
    }

    fn add_record(&mut self, event: MjaiEvent) {
        let mut d = self.data.lock().unwrap();
        d.selected_action = None;
        d.record.push(to_value(event).unwrap());
    }

    fn confirm_riichi_accepted(&mut self, stg: &Stage) {
        if let Some(s) = self.try_riichi {
            self.try_riichi = None;
            self.add_record(MjaiEvent::reach_accepted(s, &stg.get_scores()));
        }
    }

    fn notify_begin(&mut self, _stg: &Stage, _event: &EventBegin) {
        assert!(self.seat != NO_SEAT);
        self.is_new_game = true;
    }

    fn notify_new(&mut self, stg: &Stage, event: &EventNew) {
        assert!(self.seat != NO_SEAT);

        // 前の局のデータがすべて送信されていない場合は待機
        let wait = {
            let d = self.data.lock().unwrap();
            d.cursor < d.record.len()
        };
        if wait {
            sleep_ms(1000);
        }

        let mut data = SharedData::new();
        if self.is_new_game {
            data.send_start_game = true;
            self.is_new_game = false;
        }
        data.seat = self.seat;
        data.mode = event.mode;
        *self.data.lock().unwrap() = data;
        self.try_riichi = None;

        // 親番の14枚目の牌は最初のツモとして扱うので取り除く
        let mut ph = event.hands.clone();
        let d = stg.players[stg.turn].drawn.unwrap();
        vec_remove(&mut ph[stg.turn], &d);

        self.add_record(MjaiEvent::start_kyoku(
            self.seat,
            event.bakaze,
            event.kyoku,
            event.honba,
            event.kyoutaku,
            &event.doras,
            &ph,
            &stg.get_scores(),
        ));

        let event2 = EventDeal {
            seat: event.kyoku,
            tile: stg.players[event.kyoku].drawn.unwrap(),
        };
        self.notify_deal(&stg, &event2);
    }

    fn notify_deal(&mut self, stg: &Stage, event: &EventDeal) {
        self.confirm_riichi_accepted(stg);
        self.add_record(MjaiEvent::tsumo(self.seat, event.seat, event.tile));
    }

    fn notify_discard(&mut self, _stg: &Stage, event: &EventDiscard) {
        if event.is_riichi {
            self.add_record(MjaiEvent::reach(event.seat));
        }

        self.add_record(MjaiEvent::dahai(event.seat, event.tile, event.is_drawn));

        if event.is_riichi {
            self.try_riichi = Some(event.seat);
        }
    }

    fn notify_meld(&mut self, stg: &Stage, event: &EventMeld) {
        self.confirm_riichi_accepted(stg);

        self.add_record(match event.meld_type {
            MeldType::Chi => {
                let lt = stg.last_tile.unwrap();
                MjaiEvent::chi(event.seat, &event.consumed, lt.2, lt.0)
            }
            MeldType::Pon => {
                let lt = stg.last_tile.unwrap();
                MjaiEvent::pon(event.seat, &event.consumed, lt.2, lt.0)
            }
            MeldType::Minkan => {
                let lt = stg.last_tile.unwrap();
                MjaiEvent::daiminkan(event.seat, &event.consumed, lt.2, lt.0)
            }
            MeldType::Ankan => MjaiEvent::ankan(event.seat, &event.consumed),
            MeldType::Kakan => {
                let c = event.consumed[0];
                let t = c.to_normal();
                let t0 = if t.is_suit() && t.1 == 5 && c.1 != 0 {
                    Tile(t.0, 0)
                } else {
                    t
                };
                MjaiEvent::kakan(event.seat, &event.consumed, &vec![t, t, t0])
            }
        });
    }

    fn notify_kita(&mut self, _stg: &Stage, _event: &EventKita) {
        panic!();
    }

    fn notify_dora(&mut self, _stg: &Stage, event: &EventDora) {
        self.add_record(MjaiEvent::dora(event.tile));
    }

    fn notify_win(&mut self, stg: &Stage, event: &EventWin) {
        for (seat, deltas, ctx) in &event.contexts {
            self.add_record(MjaiEvent::hora(
                *seat,
                stg.turn,
                stg.last_tile.unwrap().2,
                &event.ura_doras,
                ctx,
                deltas,
                &stg.get_scores(),
            ));
        }
        self.add_record(MjaiEvent::end_kyoku());
    }

    fn notify_draw(&mut self, stg: &Stage, event: &EventDraw) {
        self.add_record(MjaiEvent::ryukyoku(
            event.type_,
            &[false; SEAT],
            &[0; SEAT],
            &stg.get_scores(),
        ));
        self.add_record(MjaiEvent::end_kyoku());
    }

    fn notify_end(&mut self, stg: &Stage, _event: &EventEnd) {
        self.add_record(MjaiEvent::end_game(&stg.get_scores()));
    }
}

impl Actor for MjaiEndpoint {
    fn init(&mut self, seat: Seat) {
        self.seat = seat;
    }

    fn select_action(&mut self, stg: &Stage, acts: &Vec<Action>) -> Action {
        // possible_actionを追加
        {
            let mut d = self.data.lock().unwrap();
            let mut mjai_acts = vec![];
            for act in acts {
                if let Some(v) = MjaiAction::from_action(stg, self.seat, act) {
                    mjai_acts.push(v);
                }
            }
            d.record.last_mut().unwrap()["possible_actions"] = to_value(mjai_acts).unwrap();
            d.selected_action = None;
            d.is_riichi = false;
        }

        // possible_actionに対する応答を待機
        let mut c = 0;
        loop {
            sleep_ms(100);
            if self.data.lock().unwrap().selected_action.is_some() {
                self.timeout_count = 0;
                break;
            }
            c += 1;
            if c == self.timeout * 10 {
                error!("possible_action timeout");
                self.timeout_count += 1;
                if self.timeout_count == 5 {
                    error!("timeout_count exceeded");
                    std::process::exit(1);
                }
                return Action::nop();
            }
        }

        let d = &mut self.data.lock().unwrap();
        let mjai_act = std::mem::replace(&mut d.selected_action, None).unwrap();

        if d.is_riichi {
            d.is_riichi = false;
            if let MjaiAction::Dahai { pai, .. } = mjai_act {
                return Action::riichi(tile_from_mjai(&pai));
            } else {
                panic!();
            }
        }

        let act = mjai_act.to_action(self.seat == stg.turn);
        // actがacts内に存在する有効な操作であるかをチェック
        match act.0 {
            ActionType::Discard => {
                if self.seat != stg.turn {
                    error!("invalid discard action");
                    return Action::nop();
                }
            }
            _ => {
                if !acts.contains(&act) {
                    error!(
                        "selected_action={:?} is not contained in possible_actions={:?}",
                        act, acts
                    );
                    return Action::nop();
                }
            }
        }
        act
    }

    fn get_config(&self) -> &Config {
        &self.config
    }
}

impl Listener for MjaiEndpoint {
    fn notify_event(&mut self, stg: &Stage, event: &Event) {
        match event {
            Event::Begin(e) => self.notify_begin(stg, e),
            Event::New(e) => self.notify_new(stg, e),
            Event::Deal(e) => self.notify_deal(stg, e),
            Event::Discard(e) => self.notify_discard(stg, e),
            Event::Meld(e) => self.notify_meld(stg, e),
            Event::Kita(e) => self.notify_kita(stg, e),
            Event::Dora(e) => self.notify_dora(stg, e),
            Event::Win(e) => self.notify_win(stg, e),
            Event::Draw(e) => self.notify_draw(stg, e),
            Event::End(e) => self.notify_end(stg, e),
        }
    }
}

#[derive(Debug)]
struct SharedData {
    send_start_game: bool,
    seat: Seat,
    record: Vec<Value>,
    cursor: usize,
    selected_action: Option<MjaiAction>,
    is_riichi: bool,
    mode: usize, // (= EventNew.mode)
}

impl SharedData {
    fn new() -> Self {
        Self {
            send_start_game: false,
            seat: NO_SEAT,
            record: vec![],
            cursor: 0,
            selected_action: None,
            is_riichi: false,
            mode: 1,
        }
    }
}

fn stream_handler(
    stream: &mut TcpStream,
    data: Arc<Mutex<SharedData>>,
    debug: bool,
) -> io::Result<()> {
    let stream2 = &mut stream.try_clone().unwrap();
    let stream3 = &mut stream.try_clone().unwrap();
    let mut send = |m: &Value| send_json(stream, m, debug);
    let mut recv = || recv_json(stream2, debug);
    let mut is_alive = || is_alive(stream3);

    // hello
    send(&json!(MjaiEvent::hello()))?;
    let v = serde_json::from_value(recv()?)?;

    if let MjaiAction::Join { name, .. } = v {
        info!("player joined: {}", name);
    } else {
        error!("first message type must be 'join'");
        return Ok(());
    }

    while data.lock().unwrap().seat == NO_SEAT {
        sleep_ms(100);
        is_alive()?;
    }

    // TODO: 座席が確定する前に,一度も自身の順番, 鳴き操作が発生せずに
    //       流局などで局が終了するとその局の情報が一切送信されない
    let mut cursor = 0;
    let mut need_start_game = true;
    loop {
        // 初期化処理
        {
            let mut d = data.lock().unwrap();
            if cursor > d.record.len() {
                // 新しく試合開始した場合はリセット
                info!("mjai reset");
                cursor = 0;
            }
            let send_start_game = d.send_start_game;
            if cursor == 0 && (send_start_game || need_start_game) {
                // start_game 新しい試合が始まった場合,またはクライアントの再接続時に送信
                need_start_game = false;
                d.send_start_game = false;
                send(&json!(MjaiEvent::start_game(d.seat, d.mode)))?;
                recv()?; // recv none
            }
        }

        let len = data.lock().unwrap().record.len();
        let mut wait_act = false;
        if cursor + 1 < len {
            send(&data.lock().unwrap().record[cursor])?;
        } else if cursor + 1 == len {
            // select_actionがpossible_actionsを追加する可能性があるので待機
            // data.lock()が開放されている必要があることに注意
            sleep_ms(100);

            let record = &data.lock().unwrap().record;
            let event = &record[cursor];
            if cursor > record.len() {
                // スリープ中にrecordがリセットされている場合
                continue;
            }
            if event["possible_actions"] != json!(null) && cursor + 1 == record.len() {
                wait_act = true;
            }
            send(&event)?;
        } else {
            sleep_ms(10);
            is_alive()?;
            continue;
        }

        cursor += 1;

        let v = serde_json::from_value(recv()?)?;
        if !wait_act {
            continue;
        }

        // possible_actionsに対する応答を処理
        if let MjaiAction::Reach { .. } = v {
            // reachは仕様が特殊なので個別に処理
            send(&json!(v))?; // send reach
            let v2 = recv()?; // recv dahai
            send(&v2)?; // send dahai
            recv()?; // recv none
            {
                let d = &mut data.lock().unwrap();
                let a = serde_json::from_value(v2)?;
                d.selected_action = Some(a);
                d.is_riichi = true;
            }

            // recordに reach -> dahai -> (reach_accepted or hora) の順で追加される
            let mut step = 0;
            loop {
                sleep_ms(10);
                is_alive()?;
                if data.lock().unwrap().record.len() == cursor {
                    continue;
                }

                let d = data.lock().unwrap();
                if d.record.len() < cursor {
                    // recordが別スレッドから初期化される可能性がある
                    continue;
                }
                let event = &d.record[cursor];
                cursor += 1;
                match event["type"].as_str().unwrap() {
                    "reach" => {
                        assert!(step == 0);
                        step = 1;
                    }
                    "dahai" => {
                        if step == 0 {
                            // 手動でリーチ判断を無視した場合
                            break;
                        }
                        step = 2;
                    }
                    "reach_accepted" => {
                        assert!(step == 2);
                        send(event)?; // send reach_accepted
                        recv()?; // recv none
                        break;
                    }
                    "hora" => {
                        assert!(step == 2);
                        cursor -= 1; // horaはここでは処理しない
                        break;
                    }
                    _ => {}
                }
            }
        } else {
            data.lock().unwrap().selected_action = Some(v);
        }

        data.lock().unwrap().cursor = cursor;
    }
}

fn send_json(stream: &mut TcpStream, value: &Value, debug: bool) -> io::Result<()> {
    stream.write((value.to_string() + "\n").as_bytes())?;
    if debug {
        println!("-> {}", value.to_string());
        flush();
    }
    Ok(())
}

fn recv_json(stream: &mut TcpStream, debug: bool) -> io::Result<Value> {
    let mut buf_read = io::BufReader::new(stream);
    let mut buf = String::new();
    buf_read.read_line(&mut buf)?;
    if debug {
        println!("<- {}", buf);
        flush();
    }

    if buf.len() == 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, ""));
    }
    serde_json::from_str(&buf[..buf.len() - 1]).or_else(|e| Err(e.into()))
}

fn is_alive(stream: &mut TcpStream) -> io::Result<()> {
    stream.set_nonblocking(true).ok();
    let res = stream.peek(&mut [0; 1024]);
    stream.set_nonblocking(false).ok();
    if let Ok(0) = res {
        Err(io::Error::new(io::ErrorKind::UnexpectedEof, ""))
    } else {
        Ok(())
    }
}
