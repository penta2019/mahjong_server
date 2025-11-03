use std::task::{Context, Poll, Waker};

use rand::prelude::*;

use super::{
    actor::Actor, common::*, listener::Listener, possible_actions::*,
    stage_controller::StageController, wall::create_wall,
};
use crate::{
    hand::{evaluate_hand_ron, evaluate_hand_tsumo},
    model::*,
    util::{
        misc::{sleep, vec_to_string},
        waiter::{Waiter, waiter_waker},
    },
};

use ActionType::*;

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
    dealer: Seat,
    honba: usize,
    riichi_sticks: usize,
    scores: [Score; SEAT],
}

impl NextRoundInfo {
    fn from_stage(stg: &Stage) -> Self {
        Self {
            round: stg.round,
            dealer: stg.dealer,
            honba: stg.honba,
            riichi_sticks: stg.riichi_sticks,
            scores: [0; SEAT],
        }
    }

    fn change_dealer(&mut self) {
        self.dealer += 1;
        if self.dealer == SEAT {
            self.dealer = 0;
            self.round += 1;
        }
    }
}

#[derive(Debug)]
pub struct MahjongEngine {
    seed: u64,               // 牌山生成用の乱数のシード値
    rng: rand::rngs::StdRng, // 乱数 (牌山生成)
    pause: f64,              // ツモ前の一時停止時間
    // ゲーム制御
    rule: Rule,
    ctrl: StageController,
    melding: Option<Action>, // 鳴き処理用
    kan_dora: Option<Tile>,  // 加槓・明槓の打牌後の槓ドラ更新用
    n_deal: usize,           // 牌山(嶺上牌を除く)からツモを行った回数
    n_kan: usize,            // 槓した回数
    n_nukidora: usize,       // 北抜きの回数
    is_suukansanra: bool,    // 四槓散了の処理フラグ
    round_result: Option<RoundResult>,
    next_round_info: NextRoundInfo,
    // 牌山
    wall: Vec<Tile>,             // 牌山全体 (=136)
    dora_wall: Vec<Tile>,        // ドラ表示牌
    ura_dora_wall: Vec<Tile>,    // 裏ドラ
    replacement_wall: Vec<Tile>, // 嶺上牌
    // 非同期制御
    waiter: Waiter,
    waker: Waker,
}

impl MahjongEngine {
    pub fn new(
        seed: u64,
        rule: Rule,
        pause: f64,
        actors: [Box<dyn Actor>; SEAT],
        listeners: Vec<Box<dyn Listener>>,
    ) -> Self {
        let ctrl = StageController::new(actors, listeners);
        let rng = rand::SeedableRng::seed_from_u64(seed);
        let next_round_info = NextRoundInfo {
            round: 0,
            dealer: 0,
            honba: 0,
            riichi_sticks: 0,
            scores: [rule.initial_score; SEAT],
        };
        let (waiter, waker) = waiter_waker();

        Self {
            seed,
            rng,
            pause,
            rule,
            ctrl,
            melding: None,
            round_result: None,
            next_round_info,
            kan_dora: None,
            n_deal: 0,
            n_kan: 0,
            n_nukidora: 0,
            is_suukansanra: false,
            wall: vec![],
            dora_wall: vec![],
            ura_dora_wall: vec![],
            replacement_wall: vec![],
            waiter,
            waker,
        }
    }

    pub fn run(&mut self) {
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
                self.check_abortive_draw();
                if self.round_result.is_some() {
                    break;
                }
            }
            self.do_event_win_draw();
        }
        self.do_event_end();
    }

    pub fn get_seed(&self) -> u64 {
        self.seed
    }

    #[inline]
    pub fn get_stage(&self) -> std::sync::RwLockReadGuard<'_, Stage> {
        self.ctrl.get_stage()
    }

    #[inline]
    fn handle_event(&mut self, event: Event) {
        self.ctrl.handle_event(&event);
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
        let last_dealer = if rule.is_sanma { 2 } else { 3 };
        if stg.round == rule.round - 1 && stg.dealer == last_dealer {
            let pl = &stg.players[last_dealer];
            if pl.rank == 0 && pl.score >= rule.settlement_score {
                return true;
            }
        }

        // round延長時の対戦終了判定 (親が1位以外でオーラスが終了した場合も含む)
        if next_round == rule.round {
            for pl in &stg.players {
                if pl.score >= rule.settlement_score {
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
        self.handle_event(Event::begin());
    }

    fn do_event_new(&mut self) {
        // 卓情報初期化
        // control
        self.melding = None;
        self.kan_dora = None;
        // count
        self.n_kan = 0;
        self.n_deal = 0;
        self.n_nukidora = 0;
        // game end
        self.is_suukansanra = false;
        self.round_result = None;
        // wall
        self.wall = vec![];
        self.dora_wall = vec![];
        self.ura_dora_wall = vec![];
        self.replacement_wall = vec![];

        // 山の初期化
        self.wall = create_wall(self.rng.next_u64(), self.rule.red5);
        // self.wall = super::wall::create_wall_debug(self.rng.next_u64(), self.rule.red5);

        // 王牌
        self.dora_wall = self.draw_tiles(5); // ドラ表示牌
        self.ura_dora_wall = self.draw_tiles(5); // 裏ドラ
        self.replacement_wall = self.draw_tiles(4); // 嶺上牌

        // プレイヤーの手牌生成
        let mut ph = [vec![], vec![], vec![], vec![]];
        for s in 0..SEAT {
            ph[s] = self.draw_tiles(13);
        }

        // ドラ表示牌
        let doras = vec![self.dora_wall[0]];

        // サイコロの目の和
        let dice = self.rng.random_range(1..=6) + self.rng.random_range(1..=6);

        let rn = &self.next_round_info;
        let event = Event::new(
            self.rule.clone(),
            rn.round,
            rn.dealer,
            rn.honba,
            rn.riichi_sticks,
            doras,
            self.ctrl.get_names(),
            rn.scores,
            ph,
            self.wall.len() - self.n_deal,
            dice,
            self.wall[self.n_deal..].to_vec(),
            self.dora_wall.clone(),
            self.ura_dora_wall.clone(),
            self.replacement_wall.clone(),
        );
        self.handle_event(event);
    }

    fn do_event_deal(&mut self) {
        if self.pause != 0.0 {
            sleep(self.pause);
        }

        let stg = self.get_stage();
        let turn = stg.turn;
        let wall_count = stg.wall_count;
        drop(stg);

        if let Some(Action { ty, .. }) = self.melding {
            match ty {
                Pon | Chi => {}
                Ankan | Minkan | Kakan => {
                    let (r, kd) = self.draw_kan_tile();
                    if ty == Ankan {
                        self.handle_event(Event::dora(kd)); // 暗槓の槓ドラは打牌前
                        self.handle_event(Event::deal(turn, r, true));
                    } else {
                        self.handle_event(Event::deal(turn, r, true));
                        self.kan_dora = Some(kd); // 明槓,加槓の槓ドラは打牌後
                    }
                    self.check_suukansanra_needed();
                }
                Nukidora => {
                    let k = self.draw_nukidora_tile();
                    self.handle_event(Event::deal(turn, k, false));
                }
                _ => panic!(),
            }
        } else {
            if wall_count > 0 {
                let s = (turn + 1) % SEAT;
                let t = self.draw_tile();
                self.handle_event(Event::deal(s, t, false));
            } else {
                self.round_result = Some(RoundResult::Draw(DrawType::Kouhaiheikyoku));
            }
        }
        assert!(self.get_stage().wall_count + self.n_deal + self.n_kan == self.wall.len());
    }

    fn do_turn_operation(&mut self) {
        // ツモ番のActionの要求
        // act: Discard, Ankan, Kakan, Riichi, Tsumo, Kyushukyuhai, Nukidora
        let stg = self.get_stage();
        let turn = stg.turn;
        let pl = &stg.players[turn];
        let tenpais =
            calc_possible_tenpai_discards(pl, get_prevalent_wind(&stg), get_seat_wind(&stg, turn));
        let acts = calc_possible_turn_actions(&stg, &self.melding, &tenpais);
        drop(stg);

        let mut cx = Context::from_waker(&self.waker);
        let mut selected_acton = self.ctrl.query_action(turn, &acts, &tenpais);
        let act = loop {
            match selected_acton.as_mut().poll(&mut cx) {
                Poll::Ready(act) => {
                    break act;
                }
                Poll::Pending => {}
            }
            self.waiter.wait();
        };

        let tp = act.ty;
        let cs = act.tiles.clone();
        self.melding = None;

        match tp {
            Discard | Riichi => {}             // 後で個別にチェック
            _ => assert!(acts.contains(&act)), // 選択されたActionが有効であることを検証
        }
        match tp {
            Nop => {
                // 打牌: ツモ切り
                let t = self.get_stage().players[turn].drawn.unwrap();
                self.handle_event(Event::discard(turn, t, true, false));
            }
            Discard => {
                // 打牌: 明示的なツモ切り以外
                let stg = self.get_stage();
                let pl = &stg.players[turn];
                let t = cs[0];

                // 捨牌がツモってきた牌でかつ手牌に1枚しかないときは自動的にツモ切りフラグを追加
                let m = if let Some(d) = pl.drawn {
                    t == d && count_tile(&pl.hand, t) == 1
                } else {
                    false
                };

                assert!(
                    !acts
                        .iter()
                        .find(|a| a.ty == Discard)
                        .unwrap()
                        .tiles
                        .contains(&t)
                );
                drop(stg);
                self.handle_event(Event::discard(turn, t, m, false))
            }
            Riichi => {
                let stg = self.get_stage();
                let pl = &stg.players[turn];
                let (t, m) = if cs.is_empty() {
                    // 明示的なツモ切りリーチ
                    (pl.drawn.unwrap(), true)
                } else {
                    let t = cs[0];
                    let m = pl.drawn == Some(t) && pl.hand[t.0][t.1] == 1;
                    (t, m)
                };
                assert!(
                    acts.iter()
                        .find(|a| a.ty == Riichi)
                        .unwrap()
                        .tiles
                        .contains(&t)
                );
                drop(stg);
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
            Nukidora => {
                self.melding = Some(act);
                self.handle_event(Event::nukidora(turn, false));
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
        let acts_list = calc_possible_call_actions(&self.get_stage(), !self.is_suukansanra);

        // プレイヤー全体でのアクション数をカウント
        let mut n_rons = 0;
        let mut n_minkan = 0;
        let mut n_pon = 0;
        let mut n_chi = 0;
        for s in 0..SEAT {
            for act in &acts_list[s] {
                match act.ty {
                    Nop => {}
                    Chi => n_chi += 1,
                    Pon => n_pon += 1,
                    Minkan => n_minkan += 1,
                    Ron => n_rons += 1,
                    _ => panic!(),
                }
            }
        }

        // 選択済みのアクション
        type Meld = Option<(Seat, Action)>;
        let mut rons = vec![];
        let mut minkan: Meld = None;
        let mut pon: Meld = None;
        let mut chi: Meld = None;

        let mut selected_actions = vec![]; // actionそのものではなくFutureであることに注意
        let mut completed_actors = vec![];
        for s in 0..SEAT {
            if acts_list[s].len() > 1 {
                selected_actions.push((s, self.ctrl.query_action(s, &acts_list[s], &[])));
            }
        }

        // 各プレイヤーのcall oprationを非同期で決定するためのmini executor
        let mut event = None;
        let mut cx = Context::from_waker(&self.waker);
        loop {
            for (s, f) in &mut selected_actions {
                if !completed_actors.contains(s) {
                    match f.as_mut().poll(&mut cx) {
                        Poll::Ready(act) => {
                            let s = *s;
                            match act.ty {
                                Nop => {}
                                Chi => chi = Some((s, act)),
                                Pon => pon = Some((s, act)),
                                Minkan => minkan = Some((s, act)),
                                Ron => rons.push(s),
                                _ => panic!(
                                    "action {} not found in {}",
                                    act,
                                    vec_to_string(&acts_list[s])
                                ),
                            }

                            for act in &acts_list[s] {
                                match act.ty {
                                    Nop => {}
                                    Chi => n_chi -= 1,
                                    Pon => n_pon -= 1,
                                    Minkan => n_minkan -= 1,
                                    Ron => n_rons -= 1,
                                    _ => panic!(),
                                }
                            }
                            completed_actors.push(s);
                        }
                        Poll::Pending => {}
                    }
                }
            }

            let mut n_priority = n_rons;
            if n_priority == 0 && !rons.is_empty() {
                self.round_result = Some(RoundResult::Ron(rons));
                break;
            }
            n_priority += n_minkan;
            if n_priority == 0
                && let Some((s, act)) = minkan
            {
                let is_pao = check_pao_for_selected_action(&self.get_stage(), s, &act);
                event = Some(Event::meld(s, MeldType::Minkan, act.tiles.clone(), is_pao));
                self.melding = Some(act);
                break;
            }
            n_priority += n_pon;
            if n_priority == 0
                && let Some((s, act)) = pon
            {
                let is_pao = check_pao_for_selected_action(&self.get_stage(), s, &act);
                event = Some(Event::meld(s, MeldType::Pon, act.tiles.clone(), is_pao));
                self.melding = Some(act);
                break;
            }
            n_priority += n_chi;
            if n_priority == 0
                && let Some((s, act)) = chi
            {
                event = Some(Event::meld(s, MeldType::Chi, act.tiles.clone(), false));
                self.melding = Some(act);
                break;
            }

            if n_priority == 0 {
                break; // すべてのActionがキャンセルされた場合はここに到達
            }

            self.waiter.wait();
        }

        // アクションをまだ選択していないActorがあったら失効通知を送る
        for (s, _) in &selected_actions {
            if !completed_actors.contains(s) {
                self.ctrl.expire_action(*s);
            }
        }

        // すべてのFutureが完了するのを待機
        while selected_actions.len() != completed_actors.len() {
            for (s, f) in &mut selected_actions {
                if !completed_actors.contains(s) {
                    match f.as_mut().poll(&mut cx) {
                        Poll::Ready(_) => completed_actors.push(*s),
                        Poll::Pending => (),
                    }
                }
            }
            self.waiter.wait();
        }

        // Futureが獲得しているReadGuardがすべて開放されてからイベントを処理
        if let Some(ev) = event {
            self.handle_event(ev);
        }
    }

    fn do_event_win_draw(&mut self) {
        let (event, mut next_round_info) = match self.round_result.take().unwrap() {
            RoundResult::Tsumo => self.round_result_tusmo(),
            RoundResult::Ron(seats) => self.round_result_ron(&seats),
            RoundResult::Draw(type_) => match type_ {
                DrawType::Kouhaiheikyoku => self.round_result_draw(),
                _ => self.round_result_abortive_draw(type_),
            },
        };

        self.handle_event(event);
        next_round_info.scores = get_scores(&self.get_stage());
        self.next_round_info = next_round_info;
    }

    fn do_event_end(&mut self) {
        self.handle_event(Event::end());
    }

    fn round_result_tusmo(&self) -> (Event, NextRoundInfo) {
        let stg = self.get_stage();
        let turn = stg.turn;
        let mut round_info = NextRoundInfo::from_stage(&stg);

        let score_ctx = evaluate_hand_tsumo(&stg, &self.ura_dora_wall).unwrap();
        let (mut ron, mut non_dealer, mut dealer) = score_ctx.points;

        let pl = &stg.players[turn];
        let mut d_scores = [0; SEAT]; // 得点変動

        // TODO: 大四喜と四槓子の包の同時発生, 包を含む2倍以上の役満時の点数計算
        if let Some(pao) = pl.pao {
            // 責任払い
            ron += stg.honba as i32 * 300;
            d_scores[pao] -= ron;
            d_scores[turn] += ron;
        } else {
            // 積み棒
            non_dealer += stg.honba as i32 * 100;
            dealer += stg.honba as i32 * 100;

            for s in 0..SEAT {
                if s != turn {
                    if !is_dealer(&stg, s) {
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
        }

        // 供託
        d_scores[turn] += stg.riichi_sticks as i32 * 1000;

        // stage情報
        round_info.riichi_sticks = 0;
        // 和了が子の場合 積み棒をリセットして親交代
        if !is_dealer(&stg, turn) {
            round_info.honba = 0;
            round_info.change_dealer();
        }

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
            is_dealer: is_dealer(&stg, turn),
            is_drawn: true,
            is_riichi: pl.riichi.is_some(),
            pao: pl.pao,
            delta_scores: d_scores,
            score_context: score_ctx,
        };
        let ura_doras = self.ura_dora_wall[0..stg.doras.len()].to_vec();
        let scores = get_scores(&stg);
        let event = Event::win(
            stg.round,
            stg.dealer,
            stg.honba,
            stg.riichi_sticks,
            stg.doras.clone(),
            ura_doras,
            self.ctrl.get_names(),
            scores,
            d_scores,
            vec![win_ctx],
        );

        (event, round_info)
    }

    fn round_result_ron(&self, seats: &[Seat]) -> (Event, NextRoundInfo) {
        let stg = self.get_stage();
        let turn = stg.turn;
        let mut round_info = NextRoundInfo::from_stage(&stg);

        // 放銃者から一番近いプレイヤー順にソート
        let mut seats_sorted = vec![];
        for s in turn + 1..turn + SEAT {
            let s = s % SEAT;
            if seats.contains(&s) {
                seats_sorted.push(s);
            }
        }

        let mut is_first = true; // 上家取り
        let mut ctxs = vec![];
        let mut total_d_scores = [0; SEAT];
        for s in seats_sorted {
            let score_ctx = evaluate_hand_ron(&stg, &self.ura_dora_wall, s).unwrap();
            let (ron, _, _) = score_ctx.points;

            let pl = &stg.players[s];
            let mut d_scores = [0; SEAT]; // 得点変動

            if let Some(pao) = pl.pao {
                // 責任払いが発生している場合,ロンの半分ずつの支払い
                d_scores[turn] -= ron / 2;
                d_scores[pao] -= ron / 2;
            } else {
                d_scores[turn] -= ron; // 直撃を受けたプレイヤー
            };
            d_scores[s] += ron; // 和了ったプレイヤー

            // 積み棒&供託(上家取り)
            if is_first {
                is_first = false;
                if let Some(pao) = pl.pao {
                    // 積み棒は責任払い優先
                    d_scores[pao] -= stg.honba as i32 * 300;
                } else {
                    d_scores[turn] -= stg.honba as i32 * 300;
                }
                d_scores[s] += stg.honba as i32 * 300;
                d_scores[s] += stg.riichi_sticks as i32 * 1000;
            }

            for s in 0..SEAT {
                total_d_scores[s] += d_scores[s];
            }

            let win_ctx = WinContext {
                seat: s,
                hand: tiles_from_tile_table(&pl.hand),
                winning_tile: stg.last_tile.unwrap().2,
                melds: pl.melds.clone(),
                is_dealer: is_dealer(&stg, s),
                is_drawn: false,
                is_riichi: pl.riichi.is_some(),
                pao: pl.pao,
                delta_scores: d_scores,
                score_context: score_ctx,
            };
            ctxs.push(win_ctx);
        }

        // stage情報
        round_info.riichi_sticks = 0;
        // 子の和了がある場合は積み棒をリセット
        if seats.iter().any(|&s| !is_dealer(&stg, s)) {
            round_info.honba = 0;
        }
        // 和了が子しかいない場合は親交代
        if seats.iter().all(|&s| !is_dealer(&stg, s)) {
            round_info.change_dealer();
        }

        let ura_doras = self.ura_dora_wall[0..stg.doras.len()].to_vec();
        let event = Event::win(
            stg.round,
            stg.dealer,
            stg.honba,
            stg.riichi_sticks,
            stg.doras.clone(),
            ura_doras,
            self.ctrl.get_names(),
            get_scores(&stg),
            total_d_scores,
            ctxs,
        );

        (event, round_info)
    }

    fn round_result_draw(&self) -> (Event, NextRoundInfo) {
        let stg = self.get_stage();
        let mut round_info = NextRoundInfo::from_stage(&stg);
        round_info.honba += 1;

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
                    if is_dealer(&stg, s_nm) {
                        nm_scores[s_nm] = 12000;
                        for s in 0..SEAT {
                            if s_nm == s {
                                d_scores[s] += 12000;
                            } else {
                                d_scores[s] -= 4000;
                            }
                        }
                    } else {
                        nm_scores[s_nm] = 8000;
                        for s in 0..SEAT {
                            if s_nm == s {
                                d_scores[s] += 8000;
                            } else {
                                if is_dealer(&stg, s) {
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

        if !tenpais[stg.dealer] {
            round_info.change_dealer();
        }

        let event = Event::draw(
            DrawType::Kouhaiheikyoku,
            stg.round,
            stg.dealer,
            stg.honba,
            self.ctrl.get_names(),
            get_scores(&stg),
            d_scores,
            nm_scores,
            hands,
        );

        (event, round_info)
    }

    fn round_result_abortive_draw(&self, type_: DrawType) -> (Event, NextRoundInfo) {
        let stg = self.get_stage();
        let mut round_info = NextRoundInfo::from_stage(&stg);
        round_info.honba += 1;

        let open_seats: Vec<Seat> = match type_ {
            DrawType::Kouhaiheikyoku | DrawType::Unknown => panic!(),
            DrawType::Suufuurenda | DrawType::Suukansanra => vec![],
            DrawType::Kyushukyuhai => vec![stg.turn],
            DrawType::Sanchaho => (0..SEAT).filter(|&s| s != stg.turn).collect(),
            DrawType::Suuchariichi => (0..SEAT).collect(),
        };
        let mut hands = [vec![], vec![], vec![], vec![]];
        for s in open_seats {
            hands[s] = tiles_from_tile_table(&stg.players[s].hand);
        }

        let event = Event::draw(
            type_,
            stg.round,
            stg.dealer,
            stg.honba,
            self.ctrl.get_names(),
            get_scores(&stg),
            [0; SEAT],
            [0; SEAT],
            hands,
        );

        (event, round_info)
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
        let (c, k) = (self.n_kan, self.n_nukidora);
        self.n_kan += 1;
        (self.replacement_wall[c + k], self.dora_wall[c + 1]) // (replacement_tile, dora_tile)
    }

    fn draw_nukidora_tile(&mut self) -> Tile {
        let (c, k) = (self.n_kan, self.n_nukidora);
        self.n_nukidora += 1;
        self.replacement_wall[c + k]
    }

    fn check_abortive_draw(&mut self) {
        self.check_suufuurenda();
        self.check_suukansanra();
        self.check_suuchariichi();
    }

    fn check_suufuurenda(&mut self) {
        let stg = self.get_stage();
        if stg.wall_count != 66 {
            return;
        }

        let mut discards = vec![];
        for pl in &stg.players {
            if !pl.melds.is_empty() || pl.discards.is_empty() {
                return;
            }
            discards.push(pl.discards[0].tile);
        }

        if discards.len() == 1 && discards[0].is_wind() {
            drop(stg);
            self.round_result = Some(RoundResult::Draw(DrawType::Suufuurenda));
        }
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
        let stg = self.get_stage();
        for s in 0..SEAT {
            for m in &stg.players[s].melds {
                let mut k = 0;
                match m.meld_type {
                    MeldType::Ankan | MeldType::Kakan | MeldType::Minkan => k += 1,
                    _ => {}
                }
                if k > 0 {
                    if k == 4 {
                        return;
                    } else {
                        drop(stg);
                        self.is_suukansanra = true;
                        return;
                    };
                }
            }
        }
    }
}
