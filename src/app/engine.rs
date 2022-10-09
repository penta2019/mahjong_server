use rand::prelude::*;

use crate::actor::*;
use crate::hand::*;
use crate::listener::*;
use crate::model::*;
use crate::tool::common::*;
use crate::tool::possible_actions::*;
use crate::tool::stage_controller::StageController;
use crate::tool::wall::*;
use crate::util::connection::TcpConnection;
use crate::util::misc::*;

use crate::{error, warn};

use ActionType::*;

// [App]
#[derive(Debug)]
pub struct EngineApp {
    seed: u64,
    rule: Rule,
    n_game: u32,
    n_thread: u32,
    write: bool,
    write_tenhou: bool,
    debug: bool,
    names: [String; SEAT], // actor names
}

impl EngineApp {
    pub fn new(args: Vec<String>) -> Self {
        let mut app = Self {
            seed: 0,
            rule: Rule {
                round: 1,
                sanma: false,
                initial_score: 25000,
                minimal_1st_score: 30000,
            },
            n_game: 0,
            n_thread: 16,
            write: false,
            write_tenhou: false,
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
                "-r" => app.rule.round = next_value(&mut it, "-r"),
                "-g" => app.n_game = next_value(&mut it, "-g"),
                "-t" => app.n_thread = next_value(&mut it, "-t"),
                "-w" => app.write = true,
                "-w-tenhou" => app.write_tenhou = true,
                "-d" => app.debug = true,
                "-0" => app.names[0] = next_value(&mut it, "-0"),
                "-1" => app.names[1] = next_value(&mut it, "-1"),
                "-2" => app.names[2] = next_value(&mut it, "-2"),
                "-3" => app.names[3] = next_value(&mut it, "-3"),
                opt => {
                    error!("unknown option: {}", opt);
                    std::process::exit(0);
                }
            }
        }

        if app.seed == 0 {
            app.seed = unixtime_now() as u64;
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
        println!();

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

        // Debug port
        let conn = TcpConnection::new("127.0.0.1:52999");
        listeners.push(Box::new(EventSender::new(Box::new(conn))));

        listeners.push(Box::new(EventPrinter::new()));
        if self.write {
            listeners.push(Box::new(EventWriter::new()));
        }
        if self.write_tenhou {
            listeners.push(Box::new(crate::listener::TenhouEventWriter::new()));
        }
        if self.debug {
            listeners.push(Box::new(Debug::new()));
        }

        let mut game = MahjongEngine::new(self.seed, self.rule.clone(), actors, listeners);
        game.run();
    }

    fn run_multiple_game(&mut self, actors: [Box<dyn Actor>; 4]) {
        use std::sync::mpsc;
        use std::{thread, time};

        let mut n_game = 0;
        let mut n_thread = 0;
        let mut n_game_end = 0;
        let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(self.seed);
        let (tx, rx) = mpsc::channel();
        let mut sum_delta_scores = [0; SEAT];
        let mut sum_ranks = [0; SEAT];
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

                let rule = self.rule.clone();
                let tx2 = tx.clone();
                thread::spawn(move || {
                    let start = time::Instant::now();
                    let mut game = MahjongEngine::new(seed, rule, shuffled_actors, vec![]);
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
                        sum_delta_scores[i] += score - game.rule.initial_score;
                        sum_ranks[i] += rank;
                        print!(", ac{}:{:5}({})", i, score, rank);
                    }
                    println!();

                    n_thread -= 1;
                    n_game_end += 1;
                }
                if n_thread < self.n_thread {
                    break;
                }
                sleep(0.01);
            }

            if n_thread == 0 && n_game == self.n_game {
                for i in 0..SEAT {
                    println!(
                        "ac{} avg_rank: {:.2}, avg_delta_score: {:6}",
                        i,
                        sum_ranks[i] as f32 / n_game as f32,
                        sum_delta_scores[i] / n_game as i32,
                    );
                }
                break;
            }
        }
    }
}

// [Engine]
#[derive(Debug)]
enum RoundResult {
    Tsumo,          // ツモ番のプレイヤーの和了
    Ron(Vec<Seat>), // ロン 和了ったプレイヤーの配列 (ロン|ダブロン|トリロン)
    Draw(DrawType),
}

#[derive(Debug)]
struct NextRoundInfo {
    round: usize,
    dealer: usize,
    honba_sticks: usize,
    riichi_sticks: usize,
    scores: [Score; SEAT],
}

#[derive(Debug)]
struct MahjongEngine {
    seed: u64,               // 牌山生成用の乱数のシード値
    rng: rand::rngs::StdRng, // 乱数 (牌山生成)
    // ゲーム制御
    rule: Rule,
    ctrl: StageController,
    melding: Option<Action>, // 鳴き処理用
    kan_dora: Option<Tile>,  // 加槓・明槓の打牌後の槓ドラ更新用
    n_deal: usize,           // 牌山(嶺上牌を除く)からツモを行った回数
    n_kan: usize,            // 槓した回数
    n_kita: usize,           // 北抜きの回数
    is_suukansanra: bool,    // 四槓散了の処理フラグ
    round_result: Option<RoundResult>,
    next_round_info: NextRoundInfo,
    // 牌山
    wall: Vec<Tile>,             // 牌山全体 (=136)
    dora_wall: Vec<Tile>,        // ドラ表示牌
    ura_dora_wall: Vec<Tile>,    // 裏ドラ
    replacement_wall: Vec<Tile>, // 嶺上牌
}

impl MahjongEngine {
    fn new(
        seed: u64,
        rule: Rule,
        actors: [Box<dyn Actor>; SEAT],
        listeners: Vec<Box<dyn Listener>>,
    ) -> Self {
        let ctrl = StageController::new(actors, listeners);
        let rng = rand::SeedableRng::seed_from_u64(seed);
        let next_round_info = NextRoundInfo {
            round: 0,
            dealer: 0,
            honba_sticks: 0,
            riichi_sticks: 0,
            scores: [rule.initial_score; SEAT],
        };

        Self {
            seed,
            rng,
            rule,
            ctrl,
            melding: None,
            round_result: None,
            next_round_info,
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
        while !self.is_game_end() {
            self.do_event_new();
            loop {
                self.do_event_deal();
                if self.round_result.is_some() {
                    break;
                }
                self.do_turn_operation();
                if self.round_result.is_some() {
                    break;
                }
                self.do_call_operation();
                if self.round_result.is_some() {
                    break;
                }
            }
            self.do_event_win_draw();
        }
        self.do_event_end();
    }

    fn is_game_end(&self) -> bool {
        let stg = self.get_stage();
        let rule = &self.rule;
        let next_round = self.next_round_info.round;

        // 飛びによる対戦終了
        for s in 0..SEAT {
            if stg.players[s].score < 0 {
                return true;
            }
        }

        // オーラスで親が所定の点数より高くかつ1位の場合はゲーム終了
        let last_dealer = if rule.sanma { 2 } else { 3 };
        if stg.round == rule.round - 1 && stg.dealer == last_dealer {
            let pl = &stg.players[last_dealer];
            if pl.rank == 0 && pl.score >= rule.minimal_1st_score {
                return true;
            }
        }

        // round延長時の対戦終了判定 (親が1位以外でオーラスが終了した場合も含む)
        if next_round == rule.round {
            for pl in &stg.players {
                if pl.score >= rule.minimal_1st_score {
                    return true;
                }
            }
        }

        // round延長(南入,西入)しても点差がつかない場合は強制終了
        if next_round > rule.round {
            return true;
        }

        // 試合続行
        false
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
        self.round_result = None;
        // wall
        self.wall = vec![];
        self.dora_wall = vec![];
        self.ura_dora_wall = vec![];
        self.replacement_wall = vec![];

        // 山の初期化
        self.wall = create_wall(self.rng.next_u64());

        // 王牌
        self.dora_wall = self.draw_tiles(5); // 槓ドラ
        self.ura_dora_wall = self.draw_tiles(5); // 裏ドラ
        self.replacement_wall = self.draw_tiles(4); // 嶺上牌

        // プレイヤーの手牌生成
        let mut ph = [vec![], vec![], vec![], vec![]];
        for s in 0..SEAT {
            ph[s] = self.draw_tiles(13);
        }

        // ドラ表示牌
        let doras = vec![self.dora_wall[0]];

        let rn = &self.next_round_info;
        let event = Event::new(
            self.rule.clone(),
            rn.round,
            rn.dealer,
            rn.honba_sticks,
            rn.riichi_sticks,
            doras,
            rn.scores,
            ph,
            self.wall.len() - self.n_deal,
        );
        self.handle_event(event);
    }

    fn do_event_deal(&mut self) {
        let stg = self.get_stage();
        let turn = stg.turn;
        if let Some(Action { action_type, .. }) = self.melding {
            match action_type {
                Pon | Chi => {}
                Ankan | Minkan | Kakan => {
                    let (r, kd) = self.draw_kan_tile();
                    if action_type == Ankan {
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
            if stg.wall_count > 0 {
                let s = (turn + 1) % SEAT;
                let t = self.draw_tile();
                self.handle_event(Event::deal(s, t));
            } else {
                self.round_result = Some(RoundResult::Draw(DrawType::Kouhaiheikyoku));
            }
        }
        assert!(self.get_stage().wall_count + self.n_deal + self.n_kan == self.wall.len());
    }

    fn do_turn_operation(&mut self) {
        // ツモ番のActionの要求
        // act: Discard, Ankan, Kakan, Riichi, Tsumo, Kyushukyuhai, Kita
        let stg = self.get_stage();
        let turn = stg.turn;
        let pl = &stg.players[turn];
        let tenpais =
            calc_possible_tenpai_discards(pl, get_prevalent_wind(stg), get_seat_wind(stg, turn));
        let acts = calc_possible_turn_actions(stg, &self.melding, &tenpais);

        let mut retry = 0;
        let act = loop {
            if let Some(act) = self.ctrl.select_action(turn, &acts, &tenpais, retry) {
                break act;
            }
            retry += 1;
            sleep(0.01);
        };

        let tp = act.action_type;
        let cs = act.tiles.clone();
        self.melding = None;

        match tp {
            Discard | Riichi => {}             // 後で個別にチェック
            _ => assert!(acts.contains(&act)), // 選択されたActionが有効であることを検証
        }

        let stg = self.get_stage();
        match tp {
            Nop => {
                // 打牌: ツモ切り
                let t = stg.players[turn].drawn.unwrap();
                self.handle_event(Event::discard(turn, t, true, false));
            }
            Discard => {
                // 打牌: ツモ切り以外
                let t = cs[0];
                assert!(!acts
                    .iter()
                    .find(|a| a.action_type == Discard)
                    .unwrap()
                    .tiles
                    .contains(&t));
                self.handle_event(Event::discard(turn, t, false, false))
            }
            Riichi => {
                let pl = &stg.players[turn];
                let (t, m) = if cs.is_empty() {
                    // 明示的なツモ切りリーチ
                    (pl.drawn.unwrap(), true)
                } else {
                    let t = cs[0];
                    let m = pl.drawn == Some(t) && pl.hand[t.0][t.1] == 1;
                    (t, m)
                };
                assert!(acts
                    .iter()
                    .find(|a| a.action_type == Riichi)
                    .unwrap()
                    .tiles
                    .contains(&t));
                self.handle_event(Event::discard(turn, t, m, true));
            }
            Ankan => {
                self.melding = Some(act);
                self.handle_event(Event::meld(turn, MeldType::Ankan, cs, false));
            }
            Kakan => {
                self.melding = Some(act);
                self.handle_event(Event::meld(turn, MeldType::Kakan, cs, false));
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
            _ => panic!("action {} not found in {}", act, vec_to_string(&acts)),
        };

        if let Some(kd) = self.kan_dora {
            self.handle_event(Event::dora(kd));
            self.kan_dora = None;
        }
    }

    fn do_call_operation(&mut self) {
        // 順番以外のプレイヤーにActionを要求
        // act: Nop, Chi, Pon, Minkan, Ron
        let mut acts_list = calc_possible_call_actions(self.get_stage(), !self.is_suukansanra);

        // プレイヤー全体でのアクション数をカウント
        let mut n_rons = 0;
        let mut n_minkan = 0;
        let mut n_pon = 0;
        let mut n_chi = 0;
        for s in 0..SEAT {
            for act in &acts_list[s] {
                match act.action_type {
                    Nop => {}
                    Chi => n_chi += 1,
                    Pon => n_pon += 1,
                    Minkan => n_minkan += 1,
                    Ron => n_rons += 1,
                    _ => panic!(),
                }
            }
        }

        // query action
        type Meld = Option<(Seat, Action)>;
        let mut rons = vec![];
        let mut minkan: Meld = None;
        let mut pon: Meld = None;
        let mut chi: Meld = None;
        let mut retry = 0;
        loop {
            for s in 0..SEAT {
                let acts = &acts_list[s];
                if acts.len() <= 1 {
                    // すでにactionを選択済み または Nopのみ
                    continue;
                }
                if let Some(act) = self.ctrl.select_action(s, acts, &vec![], retry) {
                    for act in acts {
                        match act.action_type {
                            Nop => {}
                            Chi => n_chi -= 1,
                            Pon => n_pon -= 1,
                            Minkan => n_minkan -= 1,
                            Ron => n_rons -= 1,
                            _ => panic!(),
                        }
                    }
                    match act.action_type {
                        Nop => {}
                        Chi => chi = Some((s, act)),
                        Pon => pon = Some((s, act)),
                        Minkan => minkan = Some((s, act)),
                        Ron => rons.push(s),
                        _ => panic!("action {} not found in {}", act, vec_to_string(acts)),
                    }
                    acts_list[s] = vec![];
                }
            }

            let mut n_priority = n_rons;
            if n_priority == 0 && !rons.is_empty() {
                self.round_result = Some(RoundResult::Ron(rons));
                return;
            }
            n_priority += n_minkan;
            if n_priority == 0 && minkan != None {
                let (s, act) = minkan.unwrap();
                self.handle_event(Event::meld(s, MeldType::Minkan, act.tiles.clone(), false));
                self.melding = Some(act);
                break;
            }
            n_priority += n_pon;
            if n_priority == 0 && pon != None {
                let (s, act) = pon.unwrap();
                self.handle_event(Event::meld(s, MeldType::Pon, act.tiles.clone(), false));
                self.melding = Some(act);
                break;
            }
            n_priority += n_chi;
            if n_priority == 0 && chi != None {
                let (s, act) = chi.unwrap();
                self.handle_event(Event::meld(s, MeldType::Chi, act.tiles.clone(), false));
                self.melding = Some(act);
                break;
            }

            if n_priority == 0 {
                break;
            }
            retry += 1;
            sleep(0.01);
        }

        // 途中流局の確認
        if self.round_result.is_none() {
            self.check_suufuurenda();
            self.check_suukansanra();
            self.check_suuchariichi();
        }
    }

    fn do_event_win_draw(&mut self) {
        let stg = self.get_stage();
        let mut round = stg.round;
        let mut dealer = stg.dealer;
        let mut honba_sticks = stg.honba_sticks;
        let mut riichi_sticks = stg.riichi_sticks;
        let turn = stg.turn;
        let mut need_dealer_change = false; // 親の交代
        match self.round_result.as_ref().unwrap() {
            RoundResult::Tsumo => {
                let mut d_scores = [0; SEAT]; // 得点変動

                let score_ctx = evaluate_hand_tsumo(stg, &self.ura_dora_wall).unwrap();
                let (_, mut non_dealer, mut dealer) = score_ctx.points;

                // 積み棒
                non_dealer += honba_sticks as i32 * 100;
                dealer += honba_sticks as i32 * 100;

                for s in 0..SEAT {
                    if s != turn {
                        if !is_dealer(stg, s) {
                            // 子の支払い
                            d_scores[s] -= non_dealer;
                            d_scores[turn] += non_dealer;
                        } else {
                            // 親の支払い
                            d_scores[s] -= dealer;
                            d_scores[turn] += dealer;
                        }
                    };
                }

                // 供託
                d_scores[turn] += riichi_sticks as i32 * 1000;

                // stage情報
                riichi_sticks = 0;
                // 和了が子の場合　積み棒をリセットして親交代
                if !is_dealer(stg, turn) {
                    honba_sticks = 0;
                    need_dealer_change = true;
                }

                let pl = &stg.players[turn];
                let mut h = pl.hand;
                let wt = pl.drawn.unwrap();
                // 和了牌は手牌から外す
                h[wt.0][wt.1] -= 1;
                if wt.1 == 0 {
                    h[wt.0][5] -= 1
                }
                let win_ctx = WinContext {
                    seat: turn,
                    hand: tiles_from_tile_table(&h),
                    winning_tile: wt,
                    melds: pl.melds.clone(),
                    is_dealer: is_dealer(stg, turn),
                    is_drawn: true,
                    is_riichi: pl.riichi.is_some(),
                    delta_scores: d_scores,
                    score_context: score_ctx,
                };
                let ura_doras = self.ura_dora_wall[0..stg.doras.len()].to_vec();
                self.handle_event(Event::win(
                    vec![win_ctx],
                    stg.doras.clone(),
                    ura_doras,
                    get_scores(stg),
                    d_scores,
                ));
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

                let mut ctxs = vec![];
                let mut total_d_scores = [0; SEAT];
                for &s in seats {
                    let score_ctx = evaluate_hand_ron(stg, &self.ura_dora_wall, s).unwrap();
                    let (total, _, _) = score_ctx.points;
                    let mut d_scores = [0; SEAT]; // 得点変動
                    d_scores[turn] -= total; // 直撃を受けたプレイヤー
                    d_scores[s] += total; // 和了ったプレイヤー

                    // 積み棒&供託(上家取り)
                    if s == s0 {
                        d_scores[turn] -= honba_sticks as i32 * 300;
                        d_scores[s] += honba_sticks as i32 * 300;
                        d_scores[s] += riichi_sticks as i32 * 1000;
                    }
                    for s in 0..SEAT {
                        total_d_scores[s] += d_scores[s];
                    }

                    let pl = &stg.players[s];
                    let win_ctx = WinContext {
                        seat: s,
                        hand: tiles_from_tile_table(&pl.hand),
                        winning_tile: stg.last_tile.unwrap().2,
                        melds: pl.melds.clone(),
                        is_dealer: is_dealer(stg, s),
                        is_drawn: false,
                        is_riichi: pl.riichi.is_some(),
                        delta_scores: d_scores,
                        score_context: score_ctx,
                    };
                    ctxs.push(win_ctx);
                }

                // stage情報
                riichi_sticks = 0;
                // 子の和了がある場合は積み棒をリセット
                if seats.iter().any(|&s| !is_dealer(stg, s)) {
                    honba_sticks = 0;
                }
                // 和了が子しかいない場合は親交代
                need_dealer_change = seats.iter().all(|&s| !is_dealer(stg, s));

                let ura_doras = self.ura_dora_wall[0..stg.doras.len()].to_vec();
                self.handle_event(Event::win(
                    ctxs,
                    stg.doras.clone(),
                    ura_doras,
                    get_scores(stg),
                    total_d_scores,
                ));
            }
            RoundResult::Draw(type_) => {
                match type_ {
                    DrawType::Kouhaiheikyoku => {
                        // 聴牌集計
                        let mut tenpais = [false; SEAT];
                        let mut n_tenpai = 0;
                        let mut nagashimangan = false;
                        for s in 0..SEAT {
                            let pl = &stg.players[s];
                            tenpais[s] = !pl.winning_tiles.is_empty();
                            nagashimangan |= pl.is_nagashimangan;
                            if tenpais[s] {
                                n_tenpai += 1;
                            }
                        }

                        let mut hands = [vec![], vec![], vec![], vec![]];
                        for s in 0..SEAT {
                            if tenpais[s] {
                                hands[s] = tiles_from_tile_table(&stg.players[s].hand);
                            }
                        }

                        // プレイヤーごとの得点変動
                        let mut d_scores = [0; SEAT];
                        let mut nm_scores = [0; SEAT]; // 流し満貫スコア
                        if nagashimangan {
                            // 流し満貫スコア集計
                            for s_nm in 0..SEAT {
                                if stg.players[s_nm].is_nagashimangan {
                                    if is_dealer(stg, s_nm) {
                                        nm_scores[s_nm] = 12000;
                                        for s in 0..SEAT {
                                            if s_nm != s {
                                                d_scores[s] -= 4000;
                                            }
                                        }
                                    } else {
                                        nm_scores[s_nm] = 8000;
                                        for s in 0..SEAT {
                                            if s_nm != s {
                                                if is_dealer(stg, s) {
                                                    d_scores[s] -= 4000;
                                                } else {
                                                    d_scores[s] -= 2000;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            // 流局時の聴牌人数による得点変動
                            let (pay, recv) = match n_tenpai {
                                0 => (0, 0), // 全員ノーテン
                                1 => (1000, 3000),
                                2 => (1500, 1500),
                                3 => (3000, 1000),
                                4 => (0, 0), // 全員聴牌
                                _ => panic!(),
                            };

                            for s in 0..SEAT {
                                d_scores[s] = if tenpais[s] { recv } else { -pay };
                            }
                        }

                        let event =
                            Event::draw(DrawType::Kouhaiheikyoku, hands, d_scores, nm_scores);
                        self.handle_event(event);
                        need_dealer_change = !tenpais[dealer];
                    }
                    _ => {
                        let hands = [vec![], vec![], vec![], vec![]]; // TODO
                        let event = Event::draw(*type_, hands, [0; SEAT], [0; SEAT]);
                        self.handle_event(event);
                    }
                }
                honba_sticks += 1;
            }
        }

        // 親交代
        if need_dealer_change {
            dealer += 1;
            if dealer == SEAT {
                dealer = 0;
                round += 1;
            }
        }

        let stg = self.ctrl.get_stage();

        // stage情報更新
        self.next_round_info = NextRoundInfo {
            round,
            dealer,
            honba_sticks,
            riichi_sticks,
            scores: get_scores(stg),
        };

        self.round_result = None;
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
        if stg.wall_count != 66 {
            return;
        }

        let mut discards = vec![];
        for s in 0..SEAT {
            let pl = &stg.players[s];
            if !pl.melds.is_empty() {
                return;
            }
            if pl.discards.is_empty() {
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

        self.round_result = Some(RoundResult::Draw(DrawType::Suufuurenda));
    }

    fn check_suukansanra(&mut self) {
        if self.is_suukansanra {
            self.round_result = Some(RoundResult::Draw(DrawType::Suukansanra));
        }
    }

    fn check_suuchariichi(&mut self) {
        if self.get_stage().players.iter().all(|pl| pl.is_riichi) {
            self.round_result = Some(RoundResult::Draw(DrawType::Suuchariichi));
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
                match m.meld_type {
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
