use rand::prelude::*;

use crate::actor::create_actor;
use crate::controller::*;
use crate::hand::*;
use crate::listener::*;
use crate::model::*;
use crate::util::common::*;
use crate::util::server::Server;

use crate::{error, warn};

use ActionType::*;

// [App]
pub struct EngineApp {
    seed: u64,
    mode: usize,
    n_game: u32,
    n_thread: u32,
    write: bool,
    gui_port: u32,
    debug: bool,
    names: [String; SEAT], // actor names
}

impl EngineApp {
    pub fn new(args: Vec<String>) -> Self {
        use std::process::exit;

        let mut app = Self {
            seed: 0,
            mode: 1,
            n_game: 0,
            n_thread: 16,
            write: false,
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
                "-w" => app.write = true,
                "-gui-port" => app.gui_port = next_value(&mut it, "-gui-port"),
                "-d" => app.debug = true,
                "-0" => app.names[0] = next_value(&mut it, "-0"),
                "-1" => app.names[1] = next_value(&mut it, "-1"),
                "-2" => app.names[2] = next_value(&mut it, "-2"),
                "-3" => app.names[3] = next_value(&mut it, "-3"),
                opt => {
                    error!("unknown option: {}", opt);
                    exit(0);
                }
            }
        }

        if app.seed == 0 {
            app.seed = unixtime_now();
            warn!(
                "Random seed is not specified. Unix timestamp '{}' is used as seed.",
                app.seed
            );
        }

        app
    }

    pub fn run(&mut self) {
        println!("seed: {}", self.seed);

        let actors = [
            create_actor(&self.names[0]),
            create_actor(&self.names[1]),
            create_actor(&self.names[2]),
            create_actor(&self.names[3]),
        ];
        for s in 0..SEAT {
            println!("actor{}: {:?}", s, actors[s]);
        }

        let start = std::time::Instant::now();
        if self.n_game == 0 {
            self.run_single_game(actors);
        } else {
            self.run_multiple_game(actors);
        }
        println!(
            "total elapsed time: {:8.3}sec",
            start.elapsed().as_nanos() as f32 / 1000000000.0
        );
    }

    fn run_single_game(&mut self, actors: [Box<dyn Actor>; 4]) {
        let mut listeners: Vec<Box<dyn Listener>> = vec![];
        listeners.push(Box::new(StagePrinter::new()));
        let server = Server::new_ws_server(&format!("localhost:{}", self.gui_port));
        listeners.push(Box::new(StageSender::new(server)));
        ///////////////////////////////////////////////////////////////////////
        let server = Server::new_tcp_server("localhost:12345");
        listeners.push(Box::new(EventSender::new(server)));
        ///////////////////////////////////////////////////////////////////////
        if self.write {
            listeners.push(Box::new(EventWriter::new()));
        }
        // let log = crate::convert::tenhou::TenhouLog::new();
        // listeners.push(Box::new(crate::listener::TenhouEventWriter::new(log)));
        if self.debug {
            listeners.push(Box::new(Prompt::new()));
        }

        let mut game = MahjongEngine::new(self.seed, self.mode, 25000, actors, listeners);
        game.run();
    }

    fn run_multiple_game(&mut self, actors: [Box<dyn Actor>; 4]) {
        use std::sync::mpsc;
        use std::{thread, time};

        let mode = self.mode;
        let mut n_game = 0;
        let mut n_thread = 0;
        let mut n_game_end = 0;
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
                let mut shuffled_actors: [Box<dyn Actor>; SEAT] = [
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
                    game.run();
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
enum KyokuResult {
    Tsumo,          // ツモ番のプレイヤーの和了
    Ron(Vec<Seat>), // ロン 和了ったプレイヤーの配列 (ロン|ダブロン|トリロン)
    Draw(DrawType),
}

#[derive(Debug)]
struct NextKyokuInfo {
    bakaze: usize,
    kyoku: usize,
    honba: usize,
    kyoutaku: usize,
    scores: [Score; SEAT],
}

#[derive(Debug)]
struct MahjongEngine {
    seed: u64,               // 牌山生成用の乱数のシード値
    mode: usize,             // 1: 東風戦, 2: 半荘戦, 4: 一荘戦
    initial_score: Score,    // 初期得点
    rng: rand::rngs::StdRng, // 乱数 (牌山生成)
    // ゲーム制御
    ctrl: StageController,
    melding: Option<Action>, // 鳴き処理用
    kan_dora: Option<Tile>,  // 加槓・明槓の打牌後の槓ドラ更新用
    n_deal: usize,           // 牌山(嶺上牌を除く)からツモを行った回数
    n_kan: usize,            // 槓した回数
    n_kita: usize,           // 北抜きの回数
    is_suukansanra: bool,    // 四槓散了の処理フラグ
    kyoku_result: Option<KyokuResult>,
    kyoku_next: NextKyokuInfo,
    is_end: bool,
    // 牌山
    wall: Vec<Tile>,             // 牌山全体
    dora_wall: Vec<Tile>,        // ドラ表示牌
    ura_dora_wall: Vec<Tile>,    // 裏ドラ
    replacement_wall: Vec<Tile>, // 嶺上牌
}

impl MahjongEngine {
    fn new(
        seed: u64,
        mode: usize,
        initial_score: Score,
        actors: [Box<dyn Actor>; SEAT],
        listeners: Vec<Box<dyn Listener>>,
    ) -> Self {
        let ctrl = StageController::new(actors, listeners);
        let rng = rand::SeedableRng::seed_from_u64(seed);
        let kyoku_next = NextKyokuInfo {
            bakaze: 0,
            kyoku: 0,
            honba: 0,
            kyoutaku: 0,
            scores: [initial_score; SEAT],
        };

        Self {
            seed: seed,
            mode: mode,
            initial_score: initial_score,
            rng: rng,
            ctrl: ctrl,
            melding: None,
            kyoku_result: None,
            kyoku_next: kyoku_next,
            is_end: false,
            kan_dora: None,
            n_deal: 0,
            n_kan: 0,
            n_kita: 0,
            is_suukansanra: false,
            wall: vec![],
            dora_wall: vec![],
            ura_dora_wall: vec![],
            replacement_wall: vec![],
        }
    }

    #[inline]
    fn get_stage(&self) -> &Stage {
        self.ctrl.get_stage()
    }

    #[inline]
    fn handle_event(&mut self, event: Event) {
        self.ctrl.handle_event(&event);
    }

    fn run(&mut self) {
        self.do_event_begin();
        while !self.is_end {
            self.do_event_new();
            loop {
                self.do_turn_operation();
                if let Some(_) = self.kyoku_result {
                    break;
                }
                self.do_call_operation();
                if let Some(_) = self.kyoku_result {
                    break;
                }
                self.do_event_deal();
                if let Some(_) = self.kyoku_result {
                    break;
                }
            }
            self.do_event_win_draw();
        }
        self.do_event_end();
    }

    fn do_event_begin(&mut self) {
        self.handle_event(Event::Begin(EventBegin {}));
    }

    fn do_event_new(&mut self) {
        // 卓情報初期化
        // control
        self.melding = None;
        self.kan_dora = None;
        // count
        self.n_kan = 0;
        self.n_deal = 0;
        self.n_kita = 0;
        // game end
        self.is_suukansanra = false;
        self.kyoku_result = None;
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
        ph[self.kyoku_next.kyoku].push(t);
        for s in 0..SEAT {
            ph[s].sort();
        }

        // ドラ表示牌
        let doras = vec![self.dora_wall[0]];

        let rn = &self.kyoku_next;
        let event = Event::new(
            rn.bakaze,
            rn.kyoku,
            rn.honba,
            rn.kyoutaku,
            doras,
            rn.scores,
            ph,
            self.mode,
        );
        self.handle_event(event);
    }

    fn do_turn_operation(&mut self) {
        // ツモ番のActionの要求
        // act: Discard, Ankan, Kakan, Riichi, Tsumo, Kyushukyuhai, Kita
        let stg = self.get_stage();
        let turn = stg.turn;
        let acts = calc_possible_turn_actions(stg, &self.melding);
        let act = self.ctrl.select_action(turn, &acts);
        assert!(act.0 == Discard || acts.contains(&act));
        let Action(tp, cs) = act.clone();
        self.melding = None;

        let stg = self.get_stage();
        match tp {
            Nop => {
                // 打牌: ツモ切り
                let drawn = stg.players[turn].drawn.unwrap();
                self.handle_event(Event::discard(turn, drawn, true, false));
            }
            Discard => {
                // 打牌: ツモ切り以外
                self.handle_event(Event::discard(turn, cs[0], false, false))
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
                self.handle_event(Event::discard(turn, t, m, true));
            }
            Tsumo => {
                self.kyoku_result = Some(KyokuResult::Tsumo);
            }
            Kyushukyuhai => {
                self.kyoku_result = Some(KyokuResult::Draw(DrawType::Kyushukyuhai));
            }
            Kita => {
                self.melding = Some(act);
                self.handle_event(Event::kita(turn, false));
            }
            _ => panic!("action {:?} not found in {:?}", act, acts),
        };

        if let Some(kd) = self.kan_dora {
            self.handle_event(Event::dora(kd));
            self.kan_dora = None;
        }
    }

    fn do_call_operation(&mut self) {
        // 順番以外のプレイヤーにActionを要求
        // act: Nop, Chi, Pon, Minkan, Ron
        let can_meld = self.melding == None && !self.is_suukansanra;
        let acts_list = calc_possible_call_actions(self.get_stage(), can_meld);

        // query action
        type Meld = Option<(Seat, Action)>;
        let mut rons = vec![];
        let mut minkan: Meld = None;
        let mut pon: Meld = None;
        let mut chi: Meld = None;
        for s in 0..SEAT {
            let acts = &acts_list[s];
            if acts.len() == 1 {
                // Nop
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
                _ => panic!("action {:?} not found in {:?}", act, acts),
            }
        }

        // dispatch action
        if !rons.is_empty() {
            self.kyoku_result = Some(KyokuResult::Ron(rons));
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

    fn do_event_deal(&mut self) {
        let stg = self.get_stage();
        let turn = stg.turn;
        if let Some(Action(meld_type, _)) = self.melding {
            match meld_type {
                Pon | Chi => {}
                Ankan | Minkan | Kakan => {
                    let (r, kd) = self.draw_kan_tile();
                    if meld_type == Ankan {
                        self.handle_event(Event::dora(kd)); // 暗槓の槓ドラは打牌前
                        self.handle_event(Event::deal(turn, r));
                    } else {
                        self.handle_event(Event::deal(turn, r));
                        self.kan_dora = Some(kd); // 明槓,加槓の槓ドラは打牌後
                    }
                    self.check_suukansanra_needed();
                }
                Kita => {
                    let k = self.draw_kita_tile();
                    self.handle_event(Event::deal(turn, k));
                }
                _ => panic!(),
            }
        } else {
            if stg.left_tile_count > 0 {
                let s = (turn + 1) % SEAT;
                let t = self.draw_tile();
                self.handle_event(Event::deal(s, t));
            } else {
                self.kyoku_result = Some(KyokuResult::Draw(DrawType::Kouhaiheikyoku));
            }
        }
        assert!(self.get_stage().left_tile_count + self.n_deal + self.n_kan == self.wall.len());
    }

    fn do_event_win_draw(&mut self) {
        let stg = self.get_stage();
        let mut bakaze = stg.bakaze;
        let mut kyoku = stg.kyoku;
        let mut honba = stg.honba;
        let mut kyoutaku = stg.kyoutaku;
        let turn = stg.turn;
        let mut need_leader_change = false; // 親の交代
        match self.kyoku_result.as_ref().unwrap() {
            KyokuResult::Tsumo => {
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
                self.handle_event(Event::win(ura_doras, contexts));
            }
            KyokuResult::Ron(seats) => {
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
                self.handle_event(Event::win(ura_doras, contexts));
            }
            KyokuResult::Draw(type_) => {
                match type_ {
                    DrawType::Kouhaiheikyoku => {
                        // 聴牌集計
                        let mut tenpais = [false; SEAT];
                        let mut n_tenpai = 0;
                        for s in 0..SEAT {
                            tenpais[s] = !stg.players[s].win_tiles.is_empty();
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

                        let mut hands = [vec![], vec![], vec![], vec![]];
                        for s in 0..SEAT {
                            if tenpais[s] {
                                hands[s] = tiles_from_tile_table(&stg.players[s].hand);
                            }
                        }

                        let event = Event::draw(DrawType::Kouhaiheikyoku, hands, tenpais, d_scores);
                        self.handle_event(event);
                        need_leader_change = !tenpais[kyoku];
                    }
                    _ => {
                        let hands = [vec![], vec![], vec![], vec![]]; // TODO
                        let event = Event::draw(*type_, hands, [false; SEAT], [0; SEAT]);
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
                bakaze += 1;
            }
        }

        let stg = self.ctrl.get_stage();

        // stage情報更新
        self.kyoku_next = NextKyokuInfo {
            bakaze,
            kyoku,
            honba,
            kyoutaku,
            scores: stg.get_scores(),
        };

        // 対戦終了判定
        if bakaze == self.mode {
            self.is_end = true;
        }

        // 飛びによる対戦終了
        for s in 0..SEAT {
            if stg.players[s].score < 0 {
                self.is_end = true;
            }
        }

        self.kyoku_result = None;
    }

    fn do_event_end(&mut self) {
        self.handle_event(Event::end());
    }

    fn draw_tile(&mut self) -> Tile {
        let c = self.n_deal;
        self.n_deal += 1;
        self.wall[c]
    }

    fn draw_tiles(&mut self, count: usize) -> Vec<Tile> {
        let c = self.n_deal;
        self.n_deal += count;
        self.wall[c..self.n_deal].to_vec()
    }

    fn draw_kan_tile(&mut self) -> (Tile, Tile) {
        let (c, k) = (self.n_kan, self.n_kita);
        self.n_kan += 1;
        (self.replacement_wall[c + k], self.dora_wall[c + 1]) // (replacement_tile, dora_tile)
    }

    fn draw_kita_tile(&mut self) -> Tile {
        let (c, k) = (self.n_kan, self.n_kita);
        self.n_kita += 1;
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

        if let None = self.kyoku_result {
            self.kyoku_result = Some(KyokuResult::Draw(DrawType::Suufuurenda));
        }
    }

    fn check_suukansanra(&mut self) {
        if self.is_suukansanra && self.melding == None {
            if let None = self.kyoku_result {
                self.kyoku_result = Some(KyokuResult::Draw(DrawType::Suukansanra));
            }
        }
    }

    fn check_suuchariichi(&mut self) {
        if self.get_stage().players.iter().all(|pl| pl.is_riichi) {
            if let None = self.kyoku_result {
                self.kyoku_result = Some(KyokuResult::Draw(DrawType::Suuchariichi));
            }
        }
    }

    fn check_suukansanra_needed(&mut self) {
        if self.n_kan != 4 {
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

// [Utility]
pub fn create_wall(seed: u64) -> Vec<Tile> {
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
    wall
}
