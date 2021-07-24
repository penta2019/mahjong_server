use rand::prelude::*;
use serde_json::json;

use crate::actor::create_actor;
use crate::controller::*;
use crate::hand::*;
use crate::listener::{EventWriter, StagePrinter};
use crate::model::*;
use crate::util::common::*;
use crate::util::ws_server::*;

use ActionType::*;
use StageOperation::*;

// [App]
pub struct EngineApp {
    seed: u64,
    mode: usize,
    n_game: u32,
    n_thread: u32,
    write_to_file: bool,
    gui_port: u32,
    debug: bool,
    names: [String; 4], // actor names
}

impl EngineApp {
    pub fn new(args: Vec<String>) -> Self {
        use std::process::exit;

        let mut app = Self {
            seed: 0,
            mode: 1,
            n_game: 0,
            n_thread: 16,
            write_to_file: false,
            gui_port: super::GUI_PORT,
            debug: false,
            names: [
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            ],
        };

        let mut it = args.iter();
        while let Some(s) = it.next() {
            match s.as_str() {
                "-s" => app.seed = next_value(&mut it, "-s"),
                "-m" => app.mode = next_value(&mut it, "-m"),
                "-g" => app.n_game = next_value(&mut it, "-g"),
                "-t" => app.n_thread = next_value(&mut it, "-t"),
                "-w" => app.write_to_file = true,
                "-gui-port" => app.gui_port = next_value(&mut it, "-gui-port"),
                "-d" => app.debug = true,
                "-0" => app.names[0] = next_value(&mut it, "-0"),
                "-1" => app.names[1] = next_value(&mut it, "-1"),
                "-2" => app.names[2] = next_value(&mut it, "-2"),
                "-3" => app.names[3] = next_value(&mut it, "-3"),
                opt => {
                    println!("Unknown option: {}", opt);
                    exit(0);
                }
            }
        }

        if app.seed == 0 {
            app.seed = unixtime_now();
            println!(
                "Random seed is not specified. Unix timestamp '{}' is used as seed.",
                app.seed
            );
        }

        app
    }

    pub fn run(&mut self) {
        let start = std::time::Instant::now();
        if self.n_game == 0 {
            self.run_single_game();
        } else {
            self.run_multiple_game();
        }
        println!(
            "total elapsed time: {:8.3}sec",
            start.elapsed().as_nanos() as f32 / 1000000000.0
        );
    }

    fn run_single_game(&mut self) {
        let actors = [
            create_actor(&self.names[0]),
            create_actor(&self.names[1]),
            create_actor(&self.names[2]),
            create_actor(&self.names[3]),
        ];

        let mut listeners: Vec<Box<dyn Listener>> = vec![Box::new(StagePrinter {})];
        if self.write_to_file {
            listeners.push(Box::new(EventWriter::new()));
        }
        let mut game = MahjongEngine::new(self.seed, self.mode, 25000, actors, listeners);
        let send_recv = create_ws_server(self.gui_port);

        loop {
            let stop = if let Deal = &game.next_op {
                true
            } else {
                false
            };

            let end = game.next_step();
            if let Some((s, r)) = send_recv.lock().unwrap().as_ref() {
                // 送られてきたメッセージをすべて表示
                loop {
                    match r.try_recv() {
                        Ok(msg) => {
                            println!("[WS] message: {}", msg);
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }

                // stageの状態をjsonにエンコードして送信
                let value = json!({
                    "type": "stage",
                    "data": game.get_stage(),
                });
                s.send(value.to_string()).ok();
            }

            if self.debug && stop {
                prompt();
            }

            if end {
                break;
            }
        }
    }

    fn run_multiple_game(&mut self) {
        use std::sync::mpsc;
        use std::{thread, time};

        let mode = self.mode;
        let mut n_game = 0;
        let mut n_thread = 0;
        let mut n_game_end = 0;
        let actors: [Box<dyn Actor>; 4] = [
            create_actor(&self.names[0]),
            create_actor(&self.names[1]),
            create_actor(&self.names[2]),
            create_actor(&self.names[3]),
        ];

        let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(self.seed);
        let (tx, rx) = mpsc::channel();
        let mut total_score_delta = [0; SEAT];
        let mut total_rank_sum = [0; SEAT];
        loop {
            if n_game < self.n_game && n_thread < self.n_thread {
                n_game += 1;
                n_thread += 1;

                let seed = rng.next_u64();
                let mut shuffle_table = [0, 1, 2, 3];
                shuffle_table.shuffle(&mut rng);
                let null = create_actor("Null");
                let mut shuffled_actors: [Box<dyn Actor>; 4] = [
                    null.clone_box(),
                    null.clone_box(),
                    null.clone_box(),
                    null.clone_box(),
                ];
                for s in 0..SEAT {
                    shuffled_actors[s] = actors[shuffle_table[s]].clone_box();
                }

                let tx2 = tx.clone();
                thread::spawn(move || {
                    let start = time::Instant::now();
                    let mut game = MahjongEngine::new(seed, mode, 25000, shuffled_actors, vec![]);
                    loop {
                        if game.next_step() {
                            break;
                        }
                    }

                    tx2.send((shuffle_table, game, start.elapsed())).unwrap();
                });
            }

            loop {
                if let Ok((shuffle, game, elapsed)) = rx.try_recv() {
                    let ms = elapsed.as_nanos() / 1000000;
                    print!("{:5},{:4}ms,{:20}", n_game_end, ms, game.seed);
                    for s in 0..SEAT {
                        let pl = &game.get_stage().players[s];
                        let (score, rank) = (pl.score, pl.rank + 1);
                        let i = shuffle[s];
                        total_score_delta[i] += score - game.initial_score;
                        total_rank_sum[i] += rank;
                        print!(", ac{}:{:5}({})", i, score, rank);
                    }
                    println!();

                    n_thread -= 1;
                    n_game_end += 1;
                }
                if n_thread < self.n_thread {
                    break;
                }
                sleep_ms(10);
            }

            if n_thread == 0 && n_game == self.n_game {
                for i in 0..SEAT {
                    println!(
                        "ac{} avg_rank: {:.2}, avg_score_delta: {:6}",
                        i,
                        total_rank_sum[i] as f32 / n_game as f32,
                        total_score_delta[i] / n_game as i32,
                    );
                }
                break;
            }
        }
    }
}

// [Engine]
#[derive(Debug)]
enum StageOperation {
    GameStart, // 対戦開始
    New,       // 局開始 loop {
    Turn,      //   ツモ番のプレイヤーの操作 (打牌, ツモなど)
    Call,      //   ツモ番以外のプレイヤーの操作 (鳴き, ロンなど)
    Deal,      //   ツモ(リンシャン牌を含む)
    End,       // } 局終了
    GameOver,  // 対戦終了
}

#[derive(Debug)]
enum RoundResult {
    Tsumo,          // ツモ番のプレイヤーの和了
    Ron(Vec<Seat>), // ロン 和了ったプレイヤーの配列 (ロン|ダブロン|トリロン)
    Draw(DrawType),
}

#[derive(Debug)]
struct NextRoundInfo {
    round: usize,
    kyoku: usize,
    honba: usize,
    kyoutaku: usize,
    scores: [Score; SEAT],
}

#[derive(Debug)]
struct MahjongEngine {
    seed: u64,               // 牌山生成用の乱数のシード値
    n_round: usize,          // 1: 東風戦, 2: 半荘戦, 4: 一荘戦
    initial_score: Score,    // 初期得点
    rng: rand::rngs::StdRng, // 乱数 (牌山生成)
    // ゲーム制御
    ctrl: StageController,
    next_op: StageOperation,
    melding: Option<Action>, // 鳴き処理用
    kan_dora: Option<Tile>,  // 加槓・明槓の打牌後の槓ドラ更新用
    wall_count: usize,       // 牌山からツモを行った回数
    kan_count: usize,        // 槓した回数
    kita_count: usize,       // 北抜きの回数
    is_suukansanra: bool,    // 四槓散了の処理フラグ
    round_result: Option<RoundResult>,
    round_next: NextRoundInfo,
    is_game_over: bool,
    // 牌山
    wall: Vec<Tile>,             // 牌山全体
    dora_wall: Vec<Tile>,        // ドラ表示牌
    ura_dora_wall: Vec<Tile>,    // 裏ドラ
    replacement_wall: Vec<Tile>, // 嶺上牌
}

impl MahjongEngine {
    fn new(
        seed: u64,
        n_round: usize,
        initial_score: Score,
        actors: [Box<dyn Actor>; SEAT],
        listeners: Vec<Box<dyn Listener>>,
    ) -> Self {
        let ctrl = StageController::new(actors, listeners);
        let rng = rand::SeedableRng::seed_from_u64(seed);
        let round_next = NextRoundInfo {
            round: 0,
            kyoku: 0,
            honba: 0,
            kyoutaku: 0,
            scores: [initial_score; SEAT],
        };

        Self {
            seed: seed,
            n_round: n_round,
            initial_score: initial_score,
            rng: rng,
            ctrl: ctrl,
            next_op: GameStart,
            melding: None,
            round_result: None,
            round_next: round_next,
            is_game_over: false,
            kan_dora: None,
            wall_count: 0,
            kan_count: 0,
            kita_count: 0,
            is_suukansanra: false,
            wall: vec![],
            dora_wall: vec![],
            ura_dora_wall: vec![],
            replacement_wall: vec![],
        }
    }

    #[inline]
    fn get_stage(&self) -> &Stage {
        return self.ctrl.get_stage();
    }

    #[inline]
    fn handle_event(&mut self, event: Event) {
        self.ctrl.handle_event(&event);
    }

    fn next_step(&mut self) -> bool {
        if let Some(_) = self.round_result {
            self.next_op = End;
        }
        if self.is_game_over {
            self.next_op = GameOver;
        }

        match self.next_op {
            GameStart => {
                self.do_game_start();
                self.next_op = New;
            }
            New => {
                self.do_round_new();
                self.next_op = Turn;
            }
            Turn => {
                self.do_turn_operation();
                self.next_op = Call;
            }
            Call => {
                self.do_call_operation();
                self.next_op = Deal;
            }
            Deal => {
                self.do_deal_tile();
                self.next_op = Turn;
            }
            End => {
                self.do_round_end();
                self.next_op = New;
            }
            GameOver => {
                self.do_game_result();
                return true;
            }
        }
        false
    }

    fn do_game_start(&mut self) {
        self.handle_event(Event::GameStart(EventGameStart {}));
    }

    fn do_round_new(&mut self) {
        // 卓情報初期化
        // control
        self.melding = None;
        self.kan_dora = None;
        // count
        self.kan_count = 0;
        self.wall_count = 0;
        self.kita_count = 0;
        // round end
        self.is_suukansanra = false;
        self.round_result = None;
        // wall
        self.wall = vec![];
        self.dora_wall = vec![];
        self.ura_dora_wall = vec![];
        self.replacement_wall = vec![];

        let is_3p = false;

        // 山の初期化
        self.wall = create_wall(self.rng.next_u64());

        // 王牌
        self.dora_wall = self.draw_tiles(5); // 槓ドラ
        self.ura_dora_wall = self.draw_tiles(5); // 裏ドラ
        self.replacement_wall = self.draw_tiles(if is_3p { 8 } else { 4 }); // 嶺上牌

        // プレイヤーの手牌生成
        let mut ph = [vec![], vec![], vec![], vec![]];
        for s in 0..SEAT {
            ph[s] = self.draw_tiles(13);
        }
        // 親の14枚目
        let t = self.draw_tile();
        ph[self.round_next.kyoku].push(t);
        for s in 0..SEAT {
            ph[s].sort();
        }

        // ドラ表示牌
        let doras = vec![self.dora_wall[0]];

        let rn = &self.round_next;
        let event = Event::round_new(
            rn.round,
            rn.kyoku,
            rn.honba,
            rn.kyoutaku,
            doras,
            rn.scores,
            ph,
            self.n_round,
        );
        self.handle_event(event);
    }

    fn do_turn_operation(&mut self) {
        // ツモ番のActionの要求
        // act: Discard, Ankan, Kakan, Riichi, Tsumo, Kyushukyuhai, Kita
        let stg = &self.get_stage();
        let turn = stg.turn;
        let mut acts = vec![Action::nop()];

        if !stg.players[turn].is_riichi {
            if let Some(act) = &self.melding {
                // 鳴き後に捨てられない牌を追加
                acts.push(Action(Discard, calc_prohibited_discards(act)));
            } else {
                acts.push(Action(Discard, vec![]))
            }
        }

        let can_op = match self.melding {
            None => true,
            Some(Action(tp, _)) => tp != Chi && tp != Pon,
        };
        if can_op {
            acts.append(&mut check_ankan(stg));
            acts.append(&mut check_kakan(stg));
            acts.append(&mut check_riichi(stg));
            acts.append(&mut check_tsumo(stg));
            acts.append(&mut check_kyushukyuhai(stg));
            acts.append(&mut check_kita(stg));
        }

        self.melding = None;
        let act = self.ctrl.select_action(turn, &acts);
        assert!(act.0 == Discard || acts.contains(&act));
        let Action(tp, cs) = act.clone();

        let stg = &self.get_stage();
        match tp {
            Nop => {
                // 打牌: ツモ切り
                let drawn = stg.players[turn].drawn.unwrap();
                self.handle_event(Event::discard_tile(turn, drawn, true, false));
            }
            Discard => {
                // 打牌: ツモ切り以外
                self.handle_event(Event::discard_tile(turn, cs[0], false, false))
            }
            Ankan => {
                self.melding = Some(act);
                self.handle_event(Event::meld(turn, MeldType::Ankan, cs));
            }
            Kakan => {
                self.melding = Some(act);
                self.handle_event(Event::meld(turn, MeldType::Kakan, cs));
            }
            Riichi => {
                let t = cs[0];
                let pl = &stg.players[turn];
                let m = pl.drawn == Some(t) && pl.hand[t.0][t.1] == 1;
                self.handle_event(Event::discard_tile(turn, t, m, true));
            }
            Tsumo => {
                self.round_result = Some(RoundResult::Tsumo);
            }
            Kyushukyuhai => {
                self.round_result = Some(RoundResult::Draw(DrawType::Kyushukyuhai));
            }
            Kita => {
                self.melding = Some(act);
                self.handle_event(Event::kita(turn, false));
            }
            _ => panic!("Action {:?} not found in {:?}", act, acts),
        };

        if let Some(kd) = self.kan_dora {
            self.handle_event(Event::Dora(EventDora { tile: kd }));
            self.kan_dora = None;
        }
    }

    fn do_call_operation(&mut self) {
        // 順番以外のプレイヤーにActionを要求
        // act: Nop, Chi, Pon, Minkan, Ron
        let stg = self.get_stage();
        let turn = stg.turn;
        let mut acts_list: [Vec<Action>; SEAT] = Default::default();
        for s in 0..SEAT {
            acts_list[s].push(Action::nop());
        }
        // 暗槓,加槓,四槓散了に対して他家はロン以外の操作は行えない
        if self.melding == None && !self.is_suukansanra {
            for (s, act) in check_chi(stg) {
                acts_list[s].push(act);
            }
            for (s, act) in check_pon(stg) {
                acts_list[s].push(act);
            }
            for (s, act) in check_minkan(stg) {
                acts_list[s].push(act);
            }
        }
        for (s, act) in check_ron(stg) {
            acts_list[s].push(act);
        }

        // query action
        type Meld = Option<(Seat, Action)>;
        let mut rons = vec![];
        let mut minkan: Meld = None;
        let mut pon: Meld = None;
        let mut chi: Meld = None;
        for s in 0..SEAT {
            let acts = &acts_list[s];
            if s == turn || acts.len() == 1 {
                continue;
            }
            let act = self.ctrl.select_action(s, acts);
            // calc_action_index(&acts, &act); // opがacts内に存在することを確認
            match act.0 {
                Nop => {}
                Chi => chi = Some((s, act)),
                Pon => pon = Some((s, act)),
                Minkan => minkan = Some((s, act)),
                Ron => rons.push(s),
                _ => panic!("Action {:?} not found in {:?}", act, acts),
            }
        }

        // dispatch action
        if !rons.is_empty() {
            self.round_result = Some(RoundResult::Ron(rons));
            return;
        } else if let Some((s, act)) = minkan {
            self.handle_event(Event::meld(s, MeldType::Minkan, act.1.clone()));
            self.melding = Some(act);
        } else if let Some((s, act)) = pon {
            // PonをChiより優先して処理
            self.handle_event(Event::meld(s, MeldType::Pon, act.1.clone()));
            self.melding = Some(act);
        } else if let Some((s, act)) = chi {
            self.handle_event(Event::meld(s, MeldType::Chi, act.1.clone()));
            self.melding = Some(act);
        };

        // 途中流局の確認
        self.check_suufuurenda();
        self.check_suukansanra();
        self.check_suuchariichi();
    }

    fn do_deal_tile(&mut self) {
        let stg = self.get_stage();
        let turn = stg.turn;
        if let Some(Action(meld_type, _)) = self.melding {
            match meld_type {
                Pon | Chi => {}
                Ankan | Minkan | Kakan => {
                    let (r, kd) = self.draw_kan_tile();
                    if meld_type == Ankan {
                        self.handle_event(Event::dora(kd)); // 暗槓の槓ドラは打牌前
                        self.handle_event(Event::deal_tile(turn, r));
                    } else {
                        self.handle_event(Event::deal_tile(turn, r));
                        self.kan_dora = Some(kd); // 明槓,加槓の槓ドラは打牌後
                    }
                    self.check_suukansanra_needed();
                }
                Kita => {
                    let k = self.draw_kita_tile();
                    self.handle_event(Event::deal_tile(turn, k));
                }
                _ => panic!(),
            }
        } else {
            if stg.left_tile_count > 0 {
                let s = (turn + 1) % SEAT;
                let t = self.draw_tile();
                self.handle_event(Event::deal_tile(s, t));
            } else {
                self.round_result = Some(RoundResult::Draw(DrawType::Kouhaiheikyoku));
            }
        }
        assert!(
            self.get_stage().left_tile_count + self.wall_count + self.kan_count == self.wall.len()
        );
    }

    fn do_round_end(&mut self) {
        let stg = self.get_stage();
        let mut round = stg.round;
        let mut kyoku = stg.kyoku;
        let mut honba = stg.honba;
        let mut kyoutaku = stg.kyoutaku;
        let turn = stg.turn;
        let mut need_leader_change = false; // 親の交代
        match self.round_result.as_ref().unwrap() {
            RoundResult::Tsumo => {
                let mut d_scores = [0; SEAT]; // 得点変動

                let ctx = evaluate_hand_tsumo(stg, &self.ura_dora_wall).unwrap();
                let (_, mut non_leader, mut leader) = ctx.points;

                // 積み棒
                non_leader += honba as i32 * 100;
                leader += honba as i32 * 100;

                for s in 0..SEAT {
                    if s != turn {
                        if !stg.is_leader(s) {
                            // 子の支払い
                            d_scores[s] -= non_leader;
                            d_scores[turn] += non_leader;
                        } else {
                            // 親の支払い
                            d_scores[s] -= leader;
                            d_scores[turn] += leader;
                        }
                    };
                }

                // 供託
                d_scores[turn] += kyoutaku as i32 * 1000;

                // stage情報
                kyoutaku = 0;
                // 和了が子の場合　積み棒をリセットして親交代
                if !stg.is_leader(turn) {
                    honba = 0;
                    need_leader_change = true;
                }

                let contexts = vec![(turn, d_scores, ctx)];
                let ura_doras = self.ura_dora_wall[0..stg.doras.len()].to_vec();
                self.handle_event(Event::round_end_win(ura_doras, contexts));
            }
            RoundResult::Ron(seats) => {
                // 放銃者から一番近い和了プレイヤーの探索(上家取り)
                let mut s0 = SEAT;
                for s1 in turn + 1..turn + SEAT {
                    let s2 = s1 % SEAT;
                    if seats.contains(&s2) {
                        s0 = s2;
                        break;
                    }
                }
                assert!(s0 != SEAT);

                let mut contexts = vec![];
                for &s in seats {
                    let ctx = evaluate_hand_ron(stg, &self.ura_dora_wall, s).unwrap();
                    let (total, _, _) = ctx.points;
                    let mut d_scores = [0; SEAT]; // 得点変動
                    d_scores[turn] -= total; // 直撃を受けたプレイヤー
                    d_scores[s] += total; // 和了ったプレイヤー

                    // 積み棒&供託(上家取り)
                    if s == s0 {
                        d_scores[turn] -= honba as i32 * 300;
                        d_scores[s] += honba as i32 * 300;
                        d_scores[s] += kyoutaku as i32 * 1000;
                    }

                    contexts.push((s, d_scores, ctx));
                }

                // stage情報
                kyoutaku = 0;
                // 子の和了がある場合は積み棒をリセット
                if seats.iter().any(|&s| !stg.is_leader(s)) {
                    honba = 0;
                }
                // 和了が子しかいない場合は親交代
                need_leader_change = seats.iter().all(|&s| !stg.is_leader(s));

                let ura_doras = self.ura_dora_wall[0..stg.doras.len()].to_vec();
                self.handle_event(Event::round_end_win(ura_doras, contexts));
            }
            RoundResult::Draw(draw_type) => {
                match draw_type {
                    DrawType::Kouhaiheikyoku => {
                        // 聴牌集計
                        let mut tenpais = [false; SEAT];
                        let mut n_tenpai = 0;
                        for s in 0..SEAT {
                            let h = &stg.players[s].hand;
                            tenpais[s] = !calc_tiles_to_normal_win(h).is_empty()
                                || !calc_tiles_to_chiitoitsu_win(h).is_empty()
                                || !calc_tiles_to_kokushimusou_win(h).is_empty();
                            if tenpais[s] {
                                n_tenpai += 1;
                            }
                        }

                        // 流局時の聴牌人数による得点変動
                        let (pay, recv) = match n_tenpai {
                            0 => (0, 0), // 全員ノーテン
                            1 => (1000, 3000),
                            2 => (1500, 1500),
                            3 => (3000, 1000),
                            4 => (0, 0), // 全員聴牌
                            _ => panic!(),
                        };

                        // プレイヤーごとの得点変動
                        let mut d_scores = [0; SEAT];
                        for s in 0..SEAT {
                            d_scores[s] = if tenpais[s] { recv } else { -pay };
                        }

                        self.handle_event(Event::round_end_no_tile(tenpais, d_scores));
                        need_leader_change = !tenpais[kyoku];
                    }
                    _ => {
                        let event = Event::round_end_draw(*draw_type);
                        self.handle_event(event);
                    }
                }
                honba += 1;
            }
        }

        // 親交代
        if need_leader_change {
            kyoku += 1;
            if kyoku == SEAT {
                kyoku = 0;
                round += 1;
            }
        }

        let stg = self.ctrl.get_stage();

        // stage情報更新
        self.round_next = NextRoundInfo {
            round,
            kyoku,
            honba,
            kyoutaku,
            scores: stg.get_scores(),
        };

        // 対戦終了判定
        if round == self.n_round {
            self.is_game_over = true;
        }

        // 飛びによる対戦終了
        for s in 0..SEAT {
            if stg.players[s].score < 0 {
                self.is_game_over = true;
            }
        }

        self.round_result = None;
    }

    fn do_game_result(&mut self) {
        self.handle_event(Event::game_over());
    }

    fn draw_tile(&mut self) -> Tile {
        let c = self.wall_count;
        self.wall_count += 1;
        self.wall[c]
    }

    fn draw_tiles(&mut self, count: usize) -> Vec<Tile> {
        let c = self.wall_count;
        self.wall_count += count;
        self.wall[c..self.wall_count].to_vec()
    }

    fn draw_kan_tile(&mut self) -> (Tile, Tile) {
        let (c, k) = (self.kan_count, self.kita_count);
        self.kan_count += 1;
        (self.replacement_wall[c + k], self.dora_wall[c + 1]) // (replacement_tile, dora_tile)
    }

    fn draw_kita_tile(&mut self) -> Tile {
        let (c, k) = (self.kan_count, self.kita_count);
        self.kita_count += 1;
        self.replacement_wall[c + k]
    }

    fn check_suufuurenda(&mut self) {
        let stg = self.get_stage();
        if stg.left_tile_count != 66 {
            return;
        }

        let mut discards = vec![];
        for s in 0..SEAT {
            let pl = &stg.players[s];
            if !pl.melds.is_empty() {
                return;
            }
            discards.push(pl.discards[0].tile);
        }

        let t0 = discards[0];
        if !(t0.is_wind()) {
            return;
        }

        for s in 1..SEAT {
            if t0 != discards[s] {
                return;
            }
        }

        if let None = self.round_result {
            self.round_result = Some(RoundResult::Draw(DrawType::Suufuurenda));
        }
    }

    fn check_suukansanra(&mut self) {
        if self.is_suukansanra && self.melding == None {
            if let None = self.round_result {
                self.round_result = Some(RoundResult::Draw(DrawType::Suukansanra));
            }
        }
    }

    fn check_suuchariichi(&mut self) {
        if self.get_stage().players.iter().all(|pl| pl.is_riichi) {
            if let None = self.round_result {
                self.round_result = Some(RoundResult::Draw(DrawType::Suuchariichi));
            }
        }
    }

    fn check_suukansanra_needed(&mut self) {
        if self.kan_count != 4 {
            return;
        }

        // 四槓子の聴牌判定 (四槓子の際は四槓散了にならない)
        for s in 0..SEAT {
            for m in &self.get_stage().players[s].melds {
                let mut k = 0;
                match m.type_ {
                    MeldType::Ankan | MeldType::Kakan | MeldType::Minkan => k += 1,
                    _ => {}
                }
                if k > 0 {
                    if k == 4 {
                        return;
                    } else {
                        self.is_suukansanra = true;
                        break;
                    };
                }
            }
        }
    }
}

// [Turn Action Check]
// プレイヤーのツモ番に可能な操作をチェックする
// fn(&Stage) -> Option<Action>

fn check_riichi(stg: &Stage) -> Vec<Action> {
    if stg.left_tile_count < 4 {
        return vec![];
    }

    let pl = &stg.players[stg.turn];
    if pl.is_riichi || !pl.is_menzen || pl.score < 1000 {
        return vec![];
    }

    let mut acts = vec![];
    let mut f = TileTable::default();
    let ds1 = calc_discards_to_normal_tenpai(&pl.hand);
    let ds2 = calc_discards_to_chiitoitsu_tenpai(&pl.hand);
    let ds3 = calc_discards_to_kokushimusou_tenpai(&pl.hand);
    for ds in &[ds1, ds2, ds3] {
        for &(d, _) in ds {
            if f[d.0][d.1] == 0 {
                f[d.0][d.1] += 1;
                acts.push(Action::riichi(d));
            }
        }
    }

    acts
}

fn check_tsumo(stg: &Stage) -> Vec<Action> {
    if let Some(_) = evaluate_hand_tsumo(&stg, &vec![]) {
        vec![Action::tsumo()]
    } else {
        vec![]
    }
}

fn check_ankan(stg: &Stage) -> Vec<Action> {
    if stg.left_tile_count == 0 || stg.doras.len() == 5 {
        return vec![];
    }

    let ankan = |t: Tile| {
        if t.is_suit() && t.1 == 5 {
            // 赤5を含む暗槓
            Action::ankan(vec![Tile(t.0, 0), t, t, t])
        } else {
            Action::ankan(vec![t, t, t, t])
        }
    };

    let pl = &stg.players[stg.turn];
    let mut acts = vec![];
    if pl.is_riichi {
        // リーチ中でも待ちが変わらない暗槓は可能
        if let Some(t) = pl.drawn {
            let t = t.to_normal();
            if pl.hand[t.0][t.1] == 4 {
                let mut h = pl.hand.clone();

                h[t.0][t.1] -= 1;
                let mut v1 = calc_tiles_to_normal_win(&h);
                v1.sort();

                h[t.0][t.1] -= 3;
                let mut v2 = calc_tiles_to_normal_win(&h);
                v2.sort();

                if v1 == v2 {
                    acts.push(ankan(t));
                }
            }
        }
    } else {
        for ti in 0..TYPE {
            for ni in 1..TNUM {
                if pl.hand[ti][ni] == 4 {
                    acts.push(ankan(Tile(ti, ni)));
                }
            }
        }
    }

    acts
}

fn check_kakan(stg: &Stage) -> Vec<Action> {
    if stg.left_tile_count == 0 || stg.doras.len() == 5 {
        return vec![];
    }

    let pl = &stg.players[stg.turn];
    if pl.is_riichi {
        return vec![];
    }

    let mut acts = vec![];
    for m in &pl.melds {
        if m.type_ == MeldType::Pon {
            let t = m.tiles[0].to_normal();
            if pl.hand[t.0][t.1] != 0 {
                acts.push(if t.is_suit() && t.1 == 5 && pl.hand[t.0][0] > 0 {
                    Action::kakan(Tile(t.0, 0)) // 赤5
                } else {
                    Action::kakan(t)
                });
            }
        }
    }

    acts
}

fn check_kyushukyuhai(stg: &Stage) -> Vec<Action> {
    let pl = &stg.players[stg.turn];
    if !pl.discards.is_empty() {
        return vec![];
    }

    for pl2 in &stg.players {
        if !pl2.melds.is_empty() {
            return vec![];
        }
    }

    let mut c = 0;
    for ti in 0..TZ {
        if pl.hand[ti][1] != 0 {
            c += 1;
        }
        if pl.hand[ti][9] != 0 {
            c += 1;
        }
    }
    for ni in 1..8 {
        if pl.hand[TZ][ni] != 0 {
            c += 1;
        }
    }
    if c < 9 {
        return vec![];
    }

    vec![Action::kyushukyuhai()]
}

fn check_kita(stg: &Stage) -> Vec<Action> {
    if !stg.is_3p {
        return vec![];
    }

    //　海底不可
    if stg.left_tile_count == 0 {
        return vec![];
    }

    let mut acts = vec![];
    if stg.players[stg.turn].hand[TZ][WN] != 0 {
        acts.push(Action::kita());
    }

    acts
}

// [Call Action Check]
// ツモ番のプレイヤーが打牌を行ったあとに,他のプレイヤーが可能な操作をチェックする
// fn(&Stage) -> Vec<(Seat, Action)>
// ロン以外の返り値のリストは要素が2つ以上になることはないが一貫性のためVecを返却する

fn check_chi(stg: &Stage) -> Vec<(Seat, Action)> {
    if stg.left_tile_count == 0 {
        return vec![];
    }

    let pl_turn = &stg.players[stg.turn];
    let d = pl_turn.discards.last().unwrap().tile.to_normal();
    if d.is_hornor() {
        return vec![];
    }

    let s = (stg.turn + 1) % SEAT;
    if stg.players[s].is_riichi {
        return vec![];
    }

    let mut check: Vec<(Tnum, Tnum)> = vec![];
    let i = d.1;
    // l2 l1 c0(discarded) r1 r2
    let l2 = if i >= 2 { i - 2 } else { 255 };
    let l1 = if i >= 1 { i - 1 } else { 255 };
    let c0 = i;
    let r1 = i + 1;
    let r2 = i + 2;

    if 3 <= c0 {
        check.push((l2, l1));
        // red 5
        if l2 == 5 {
            check.push((0, l1));
        }
        if l1 == 5 {
            check.push((l2, 0));
        }
    }

    if c0 <= 7 {
        check.push((r1, r2));
        // red 5
        if r1 == 5 {
            check.push((0, r2));
        }
        if r2 == 5 {
            check.push((r1, 0));
        }
    }

    if 2 <= c0 && c0 <= 8 {
        check.push((l1, r1));
        // red 5
        if l1 == 5 {
            check.push((0, r1));
        }
        if r1 == 5 {
            check.push((l1, 0));
        }
    }

    let h = &stg.players[s].hand[d.0];
    let mut acts = vec![];
    for pair in check {
        if h[pair.0] > 0 && h[pair.1] > 0 {
            acts.push((s, Action::chi(vec![Tile(d.0, pair.0), Tile(d.0, pair.1)])));
        }
    }

    acts
}

fn check_pon(stg: &Stage) -> Vec<(Seat, Action)> {
    if stg.left_tile_count == 0 {
        return vec![];
    }

    let d = stg.last_tile.unwrap().2;
    let t = d.to_normal();
    let mut acts = vec![];
    for s in 0..SEAT {
        let pl = &stg.players[s];
        if pl.hand[t.0][t.1] < 2 || stg.turn == s || pl.is_riichi {
            continue;
        }

        let t0 = Tile(t.0, 0);
        let pon = Action::pon(vec![t, t]);
        let pon0 = Action::pon(vec![t0, t]); // 手牌の赤5を含むPon
        if t.is_suit() && t.1 == 5 && pl.hand[t.0][0] > 0 {
            // 赤5がある場合
            if pl.hand[t.0][t.1] > 2 {
                acts.push((s, pon));
                acts.push((s, pon0));
            } else {
                acts.push((s, pon0));
            }
        } else {
            // 5以外または赤なし
            acts.push((s, pon));
        }
    }
    acts
}

fn check_minkan(stg: &Stage) -> Vec<(Seat, Action)> {
    if stg.left_tile_count == 0 || stg.doras.len() == 5 {
        return vec![];
    }

    let d = stg.last_tile.unwrap().2;
    let t = d.to_normal();
    let mut acts = vec![];
    for s in 0..SEAT {
        let pl = &stg.players[s];
        if pl.hand[t.0][t.1] != 3 || stg.turn == s || pl.is_riichi {
            continue;
        }

        let cs = if t.is_suit() && t.1 == 5 && pl.hand[t.0][0] > 0 {
            Action::minkan(vec![Tile(t.0, 0), t, t])
        } else {
            Action::minkan(vec![t, t, t])
        };
        acts.push((s, cs));
    }
    acts
}

fn check_ron(stg: &Stage) -> Vec<(Seat, Action)> {
    let mut acts = vec![];
    for s in 0..SEAT {
        if let Some(_) = evaluate_hand_ron(stg, &vec![], s) {
            acts.push((s, Action::ron()));
        }
    }
    acts
}

// [Utility]
fn create_wall(seed: u64) -> Vec<Tile> {
    let mut wall = Vec::new();
    for ti in 0..TYPE {
        for ni in 1..TNUM {
            if ti == TZ && ni > DR {
                break;
            }
            for n in 0..TILE {
                let ni2 = if ti != TZ && ni == 5 && n == 0 { 0 } else { ni }; // 赤5
                wall.push(Tile(ti, ni2));
            }
        }
    }

    let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(seed);
    wall.shuffle(&mut rng);

    return wall;
}

fn calc_prohibited_discards(act: &Action) -> Vec<Tile> {
    let mut v = vec![];
    let Action(tp, cs) = act;
    match tp {
        Chi => {
            // 赤5が混じっている可能性を考慮
            let (t0, t1) = (cs[0].to_normal(), cs[1].to_normal());
            let ti = t0.0;
            let ni0 = t0.1;
            let ni1 = t1.1;
            let s = std::cmp::min(ni0, ni1);
            let b = std::cmp::max(ni0, ni1);
            if s + 1 == b {
                // リャンメン・ペンチャン
                let i = s - 1;
                if 0 < i {
                    v.push(Tile(ti, i));
                }
                let i = b + 1;
                if i < TNUM {
                    v.push(Tile(ti, i))
                }
            } else {
                // カンチャン
                let i = s + 1;
                v.push(Tile(ti, i));
            }
        }
        Pon => {
            v.push(cs[0].to_normal());
        }
        _ => return vec![],
    }

    let mut has5 = false;
    for t in &v {
        if t.is_suit() && t.1 == 5 {
            has5 = true;
        }
    }
    if has5 {
        v.push(Tile(v[0].0, 0));
    }

    v
}
