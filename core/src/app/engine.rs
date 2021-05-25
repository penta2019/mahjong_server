use rand::prelude::*;
use serde_json::json;

use crate::hand::evaluate::*;
use crate::hand::win::*;
use crate::model::*;
use crate::util::common::*;
use crate::util::operator::*;
use crate::util::stage_listener::*;
use crate::util::ws_server::*;

use PlayerOperation::*;
use StageOperation::*;
use TileStateType::*;

#[derive(Debug)]
enum StageOperation {
    New,      // 局開始 loop {
    Turn,     //   ツモ番のプレイヤーの操作 (打牌, ツモなど)
    Call,     //   ツモ番以外のプレイヤーの操作 (鳴き, ロンなど)
    Deal,     //   ツモ(リンシャン牌を含む)
    End,      // } 局終了
    GameOver, // 対戦終了
}

#[derive(Debug)]
enum RoundResult {
    Tsumo,          // Tsumo
    Ron(Vec<Seat>), // ロン 和了ったプレイヤーの配列 (ロン|ダブロン|トリロン)
    Draw(DrawType),
}

#[derive(Debug)]
struct Config {
    seed: u64,                            // 牌山生成用の乱数のシード値
    n_round: usize,                       // 1: 東風戦, 2: 半荘戦, 4: 一荘戦
    initial_score: i32,                   // 初期得点
    operators: [Box<dyn Operator>; SEAT], // プレイヤーまたはアルゴ
    listeners: Vec<Box<dyn StageListener>>,
}

// Stageの更新を行うと同時に更新内容をlistenersすべてに通知するマクロ
macro_rules! op {
    ($self:expr, $name:ident, $($args:expr),*) => {
        paste::item! {
            $self.stage.[< op_ $name>]($($args),*);
            for l in &mut $self.config.listeners {
                l.[<notify_op_ $name>](&$self.stage, $($args),*);
            }
        }
    };
}

#[derive(Debug)]
struct MahjongEngine {
    config: Config,
    rng: rand::rngs::StdRng, // 乱数 (牌山生成)
    // ゲーム制御
    stage: Stage,
    next_op: StageOperation,
    melding: Option<PlayerOperation>, // 鳴き処理用
    kan_dora: Option<Tile>,           // 加槓・明槓の打牌後の槓ドラ更新用
    wall_count: usize,                // 牌山からツモを行った回数
    kan_count: usize,                 // 槓した回数
    kita_count: usize,                // 北抜きの回数
    is_suukansanra: bool,             // 四槓散了の処理フラグ
    round_result: Option<RoundResult>,
    is_game_over: bool,
    // 牌山
    wall: Vec<Tile>,             // 牌山全体
    dora_wall: Vec<Tile>,        // ドラ表示牌
    ura_dora_wall: Vec<Tile>,    // 裏ドラ
    replacement_wall: Vec<Tile>, // 嶺上牌
}

impl MahjongEngine {
    fn new(config: Config) -> Self {
        let mut stg = Stage::new();
        for s in 0..SEAT {
            stg.players[s].score = config.initial_score;
        }
        let rng = rand::SeedableRng::seed_from_u64(config.seed);
        Self {
            config: config,
            rng: rng,
            stage: stg,
            next_op: New,
            melding: None,
            round_result: None,
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

    fn next_step(&mut self) -> bool {
        if let Some(_) = self.round_result {
            self.next_op = End;
        }
        if self.is_game_over {
            self.next_op = GameOver;
        }

        match self.next_op {
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

        // 古い卓から必要な情報を抽出
        let stg = &self.stage;
        let round = stg.round;
        let hand = stg.hand;
        let ben = stg.ben;
        let riichi_sticks = stg.riichi_sticks;
        let is_3p = stg.is_3p;
        let mut scores = [0; 4];
        for s in 0..SEAT {
            scores[s] = stg.players[s].score;
        }

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
        ph[hand].push(t);

        // ドラ表示牌
        let doras = vec![self.dora_wall[0]];

        op!(
            self,
            roundnew,
            round,
            hand,
            ben,
            riichi_sticks,
            &doras,
            &scores,
            &ph
        );
    }

    fn do_turn_operation(&mut self) {
        // ツモ番のPlayerOperationの要求
        // op: Discard, Ankan, Kakan, Riichi, Tsumo, Kyushukyuhai, Kita
        let stg = &self.stage;
        let turn = stg.turn;
        let mut ops = vec![Nop];
        if !stg.players[turn].is_riichi {
            // 鳴き後に捨てられない牌を追加
            ops.push(Discard(calc_prohibited_discards(&self.melding)));
        }

        match &self.melding {
            Some(Chii(_)) | Some(Pon(_)) => {} // チー・ポン後は打牌のみ
            _ => {
                if let Some(op) = check_ankan(stg) {
                    ops.push(op);
                }
                if let Some(op) = check_kakan(stg) {
                    ops.push(op);
                }
                if let Some(op) = check_riichi(stg) {
                    ops.push(op);
                }
                if let Some(op) = check_tsumo(stg) {
                    ops.push(op);
                }
                if let Some(op) = check_kyushukyuhai(stg) {
                    ops.push(op);
                }
                if let Some(op) = check_kita(stg) {
                    ops.push(op);
                }
            }
        }

        self.melding = None;
        let op = self.config.operators[turn].handle_operation(&stg, turn, &ops);
        calc_operation_index(&ops, &op); // opがops内に存在することを確認
        match &op {
            Nop => {
                // ツモ切り
                let t = self.stage.players[turn].drawn.unwrap();
                op!(self, discardtile, turn, t, true, false);
            }
            Discard(v) => {
                op!(self, discardtile, turn, v[0], false, false);
            }
            Ankan(v) => {
                op!(self, ankankakan, turn, MeldType::Ankan, v[0]);
                self.melding = Some(op);
            }
            Kakan(v) => {
                op!(self, ankankakan, turn, MeldType::Kakan, v[0]);
                self.melding = Some(op);
            }
            Riichi(v) => {
                let t = v[0];
                let pl = &self.stage.players[turn];
                let m = pl.drawn == Some(t) && pl.hand[t.0][t.1] == 1;
                op!(self, discardtile, turn, t, m, true);
            }
            Tsumo => {
                self.round_result = Some(RoundResult::Tsumo);
            }
            Kyushukyuhai => {
                self.round_result = Some(RoundResult::Draw(DrawType::Kyushukyuhai));
            }
            Kita => {
                op!(self, kita, turn, false);
                self.melding = Some(Kita);
            }
            op2 => panic!("Operation '{:?}' not found in {:?}", op2, ops),
        }

        if let Some(kd) = self.kan_dora {
            self.stage.add_dora(kd);
            self.kan_dora = None;
        }
    }

    fn do_call_operation(&mut self) {
        // 順番以外のプレイヤーにPlayerOperationを要求
        // op: Nop, Chii, Pon, Minkan, Ron
        let stg = &self.stage;
        let turn = stg.turn;
        let mut ops_list: [Vec<PlayerOperation>; SEAT] = Default::default();
        for s in 0..SEAT {
            ops_list[s].push(Nop);
        }
        // 暗槓,加槓,四槓散了に対して他家はロン以外の操作は行えない
        if self.melding == None && !self.is_suukansanra {
            for (s, op) in check_chii(stg) {
                ops_list[s].push(op);
            }
            for (s, op) in check_pon(stg) {
                ops_list[s].push(op);
            }
            for (s, op) in check_minkan(stg) {
                ops_list[s].push(op);
            }
        }
        for (s, op) in check_ron(stg) {
            ops_list[s].push(op);
        }

        // query operation
        let pl = &stg.players[turn];
        let d = pl.discards.last().unwrap().tile;
        let mut rons = vec![];
        let mut minkan: Option<(Seat, Tile)> = None;
        let mut pon: Option<(Seat, Tile, Tile)> = None;
        let mut chii: Option<(Seat, Tile, Tile)> = None;
        let mut tiles = vec![];
        let mut froms = vec![];
        for s in 0..SEAT {
            let ops = &ops_list[s];
            if s == turn || ops.len() == 1 {
                continue;
            }
            let op = self.config.operators[s].handle_operation(&stg, s, ops);
            calc_operation_index(&ops, &op); // opがops内に存在することを確認
            match &op {
                Nop => {}
                Chii(v) => chii = Some((s, v[0].0, v[0].1)),
                Pon(v) => pon = Some((s, v[0].0, v[0].1)),
                Minkan(v) => minkan = Some((s, v[0])),
                Ron => rons.push(s),
                op2 => panic!("Operation '{:?}' not found in {:?}", op2, ops),
            }
        }

        // dispatch operation
        if !rons.is_empty() {
            self.round_result = Some(RoundResult::Ron(rons));
        } else if let Some((s, t0)) = minkan {
            tiles.append(&mut vec![d, t0, t0]);
            if d.0 != TZ && d.1 == 5 && self.stage.players[s].hand[d.0][0] > 0 {
                // 手牌に赤5がある場合
                tiles.push(Tile(d.0, 0));
            } else {
                // 5以外の牌か捨て牌が赤5
                tiles.push(t0);
            }
            froms.append(&mut vec![turn, s, s, s]);
            op!(self, chiiponkan, s, MeldType::Minkan, &tiles, &froms);
            self.melding = Some(Minkan(vec![t0]));
        } else if let Some((s, t0, t1)) = pon {
            // PonをChiiより優先して処理
            tiles.append(&mut vec![d, t0, t1]);
            froms.append(&mut vec![turn, s, s]);
            op!(self, chiiponkan, s, MeldType::Pon, &tiles, &froms);
            self.melding = Some(Pon(vec![(t0, t1)]));
        } else if let Some((s, t0, t1)) = chii {
            tiles.append(&mut vec![d, t0, t1]);
            froms.append(&mut vec![turn, s, s]);
            op!(self, chiiponkan, s, MeldType::Chii, &tiles, &froms);
            self.melding = Some(Chii(vec![(t0, t1)]));
        }

        // 途中流局の確認
        self.check_suufuurenda();
        self.check_suukansanra();
        self.check_suuchariichi();
    }

    fn do_deal_tile(&mut self) {
        let turn = self.stage.turn;
        if let Some(m) = &self.melding {
            match m {
                Pon(_) | Chii(_) => {}
                Ankan(_) => {
                    let (r, kd) = self.draw_kan_tile();
                    op!(self, dealtile, turn, Some(r));
                    self.stage.add_dora(kd); // 槓ドラは打牌前
                    self.check_suukansanra_needed();
                }
                Minkan(_) => {
                    let (r, kd) = self.draw_kan_tile();
                    op!(self, dealtile, turn, Some(r));
                    self.kan_dora = Some(kd); // 槓ドラは打牌後
                    self.check_suukansanra_needed();
                }
                Kakan(_) => {
                    let (r, kd) = self.draw_kan_tile();
                    op!(self, dealtile, turn, Some(r));
                    self.kan_dora = Some(kd); // 槓ドラは打牌後
                    self.check_suukansanra_needed();
                }
                Kita => {
                    let k = self.draw_kita_tile();
                    op!(self, dealtile, turn, Some(k));
                }
                _ => panic!(),
            }
        } else {
            if self.stage.left_tile_count > 0 {
                let s = (turn + 1) % SEAT;
                let t = self.draw_tile();
                op!(self, dealtile, s, Some(t));
            } else {
                self.round_result = Some(RoundResult::Draw(DrawType::Kouhaiheikyoku));
            }
        }
        assert!(self.stage.left_tile_count + self.wall_count + self.kan_count == self.wall.len());
    }

    fn do_round_end(&mut self) {
        let stg = &self.stage;
        let mut round = stg.round;
        let mut hand = stg.hand;
        let mut ben = stg.ben;
        let mut riichi_sticks = stg.riichi_sticks;
        let turn = stg.turn;
        let mut need_leader_change = false; // 親の交代
        match self.round_result.as_ref().unwrap() {
            RoundResult::Tsumo => {
                let mut d_scores = [0; SEAT]; // 得点変動

                let score = evaluate_hand_tsumo(stg, &self.ura_dora_wall).unwrap();
                let (_, mut non_leader, mut leader) = score.pay_scores;

                // 積み棒
                non_leader += ben as i32 * 100;
                leader += ben as i32 * 100;

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
                d_scores[turn] += riichi_sticks as i32 * 1000;

                // stage情報
                riichi_sticks = 0;
                // 和了が子の場合　積み棒をリセットして親交代
                if !stg.is_leader(turn) {
                    ben = 0;
                    need_leader_change = true;
                }

                let contexts = vec![(turn, score)];
                let ura_doras = self.ura_dora_wall[0..stg.doras.len()].to_vec();
                op!(self, roundend_win, &ura_doras, &contexts, &d_scores);
            }
            RoundResult::Ron(seats) => {
                let mut d_scores = [0; SEAT]; // 得点変動

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

                    let (total, _, _) = ctx.pay_scores;
                    d_scores[turn] -= total; // 直撃を受けたプレイヤー
                    d_scores[s] += total; // 和了ったプレイヤー

                    // 積み棒&供託(上家取り)
                    if s == s0 {
                        d_scores[turn] -= ben as i32 * 300;
                        d_scores[s] += ben as i32 * 300;
                        d_scores[s] += riichi_sticks as i32 * 1000;
                    }

                    contexts.push((s, ctx));
                }

                // stage情報
                riichi_sticks = 0;
                // 子の和了がある場合は積み棒をリセット
                if seats.iter().any(|&s| !stg.is_leader(s)) {
                    ben = 0;
                }
                // 和了が子しかいない場合は親交代
                need_leader_change = seats.iter().all(|&s| !stg.is_leader(s));

                let ura_doras = self.ura_dora_wall[0..stg.doras.len()].to_vec();
                op!(self, roundend_win, &ura_doras, &contexts, &d_scores);
            }
            RoundResult::Draw(draw_type) => {
                match draw_type {
                    DrawType::Kouhaiheikyoku => {
                        // 聴牌集計
                        let mut is_ready = [false; SEAT];
                        let mut n_ready = 0;
                        for s in 0..SEAT {
                            let h = &stg.players[s].hand;
                            is_ready[s] = !calc_tiles_to_normal_win(h).is_empty()
                                || !calc_tiles_to_chiitoitsu_win(h).is_empty()
                                || !calc_tiles_to_kokushimusou_win(h).is_empty();
                            if is_ready[s] {
                                n_ready += 1;
                            }
                        }

                        // 流局時の聴牌人数による得点変動
                        let (pay, recv) = match n_ready {
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
                            d_scores[s] = if is_ready[s] { recv } else { -pay };
                        }

                        op!(self, roundend_notile, &is_ready, &d_scores);
                        need_leader_change = !is_ready[hand];
                    }
                    _ => {
                        op!(self, roundend_draw, *draw_type);
                    }
                }
                ben += 1;
            }
        }

        // 親交代
        if need_leader_change {
            hand += 1;
            if hand == SEAT {
                hand = 0;
                round += 1;
            }
        }

        // stage情報更新
        let stg = &mut self.stage;
        stg.round = round;
        stg.hand = hand;
        stg.ben = ben;
        stg.riichi_sticks = riichi_sticks;

        // 対戦終了判定
        if stg.round == self.config.n_round {
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

    fn do_game_result(&mut self) {}

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
        if self.stage.left_tile_count != 66 {
            return;
        }

        let mut discards = vec![];
        for s in 0..SEAT {
            let pl = &self.stage.players[s];
            if !pl.melds.is_empty() {
                return;
            }
            discards.push(pl.discards[0].tile);
        }

        let t0 = discards[0];
        if !(t0.0 == TZ && t0.1 <= WN) {
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
        if self.stage.players.iter().all(|pl| pl.is_riichi) {
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
            for m in &self.stage.players[s].melds {
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
                    };
                }
            }
        }
    }
}

// Turn Operation Check =======================================================
// プレイヤーのツモ番に可能な操作をチェックする
// fn(&Stage) -> Option<PlayerOperation>

fn check_riichi(stg: &Stage) -> Option<PlayerOperation> {
    if stg.left_tile_count < 4 {
        return None;
    }

    let pl = &stg.players[stg.turn];
    if pl.is_riichi || !pl.is_menzen || pl.score < 1000 {
        return None;
    }

    let mut v = vec![];
    let mut f = TileTable::default();
    let ds1 = calc_discards_to_normal_ready(&pl.hand);
    let ds2 = calc_discards_to_chiitoitsu_ready(&pl.hand);
    let ds3 = calc_discards_to_kokushimusou_ready(&pl.hand);
    for ds in &[ds1, ds2, ds3] {
        for &(d, _) in ds {
            if f[d.0][d.1] == 0 {
                f[d.0][d.1] += 1;
                v.push(d);
            }
        }
    }

    if v.is_empty() {
        return None;
    }

    Some(Riichi(v))
}

fn check_tsumo(stg: &Stage) -> Option<PlayerOperation> {
    if let Some(_) = evaluate_hand_tsumo(&stg, &vec![]) {
        Some(Tsumo)
    } else {
        None
    }
}

fn check_ankan(stg: &Stage) -> Option<PlayerOperation> {
    if stg.left_tile_count == 0 || stg.doras.len() == 5 {
        return None;
    }

    let pl = &stg.players[stg.turn];

    let mut v = vec![];
    if pl.is_riichi {
        // リーチ中でも待ちが変わらない暗槓は可能
        if let Some(t) = pl.drawn {
            if pl.hand[t.0][t.1] == 4 {
                let mut h = pl.hand.clone();
                h[t.0][t.1] -= 1;
                let mut v1 = calc_tiles_to_normal_win(&h);
                v1.sort();

                h[t.0][t.1] -= 3;
                let mut v2 = calc_tiles_to_normal_win(&h);
                v2.sort();

                if v1 == v2 {
                    v.push(t);
                }
            }
        }
    } else {
        for ti in 0..TYPE {
            for ni in 1..TNUM {
                if pl.hand[ti][ni] == 4 {
                    v.push(Tile(ti, ni));
                }
            }
        }
    }

    if !v.is_empty() {
        return Some(Ankan(v));
    }

    None
}

fn check_kakan(stg: &Stage) -> Option<PlayerOperation> {
    if stg.left_tile_count == 0 || stg.doras.len() == 5 {
        return None;
    }

    let pl = &stg.players[stg.turn];
    if pl.is_riichi {
        return None;
    }

    let mut v = vec![];
    for m in &pl.melds {
        if m.type_ == MeldType::Pon {
            let t = m.tiles[0];
            if pl.hand[t.0][t.n()] != 0 {
                if t.n() == 5 && pl.hand[t.0][0] != 0 {
                    v.push(Tile(t.0, 0)); // 赤5
                } else {
                    v.push(t);
                }
            }
        }
    }

    if !v.is_empty() {
        return Some(Kakan(v));
    }

    None
}

fn check_kyushukyuhai(stg: &Stage) -> Option<PlayerOperation> {
    let pl = &stg.players[stg.turn];
    if !pl.discards.is_empty() {
        return None;
    }

    for pl2 in &stg.players {
        if !pl2.melds.is_empty() {
            return None;
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
        return None;
    }

    Some(Kyushukyuhai)
}

fn check_kita(stg: &Stage) -> Option<PlayerOperation> {
    if !stg.is_3p {
        return None;
    }

    //　海底不可
    if stg.left_tile_count == 0 {
        return None;
    }

    let pl = &stg.players[stg.turn];
    if pl.is_riichi {
        return None;
    }

    if pl.hand[TZ][WN] != 0 {
        return Some(Kita);
    }

    None
}

// Call Operation Check =======================================================
// ツモ番のプレイヤーが打牌を行ったあとに、他のプレイヤーが可能な操作をチェックする
// fn(&Stage) -> Vec<(Seat, PlayerOperation)>
// ロン以外の返り値のリストは要素が2つ以上になることはないが一貫性のためVecを返却する

fn check_chii(stg: &Stage) -> Vec<(Seat, PlayerOperation)> {
    if stg.left_tile_count == 0 {
        return vec![];
    }

    let pl_turn = &stg.players[stg.turn];
    let d = pl_turn.discards.last().unwrap().tile;
    if d.0 == TZ {
        return vec![];
    }

    let s = (stg.turn + 1) % SEAT;
    if stg.players[s].is_riichi {
        return vec![];
    }

    let mut check: Vec<(Tnum, Tnum)> = vec![];
    let i = d.n();
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
    let mut ops = vec![];
    for pair in check {
        if h[pair.0] > 0 && h[pair.1] > 0 {
            ops.push((Tile(d.0, pair.0), Tile(d.0, pair.1)));
        }
    }

    if ops.len() != 0 {
        return vec![(s, Chii(ops))];
    }
    vec![]
}

fn check_pon(stg: &Stage) -> Vec<(Seat, PlayerOperation)> {
    if stg.left_tile_count == 0 {
        return vec![];
    }

    let d = stg.players[stg.turn].discards.last().unwrap().tile;
    let mut count = [0; SEAT];
    for &e in &stg.tile_states[d.0][d.1] {
        if let H(s) = e {
            count[s] += 1;
        }
    }
    for s in 0..SEAT {
        let pl = &stg.players[s];
        if count[s] < 2 || stg.turn == s || pl.is_riichi {
            continue;
        }

        let t0 = Tile(d.0, 0);
        let pon = (d, d);
        let pon0 = (t0, d); // 手牌の赤5を含むPon
        if d.0 != TZ && d.1 == 5 && pl.hand[d.0][0] > 0 {
            // 赤5がある場合
            if count[s] == 2 {
                return vec![(s, Pon(vec![pon0]))];
            } else {
                return vec![(s, Pon(vec![pon, pon0]))];
            }
        } else {
            // 5以外または赤なし
            return vec![(s, Pon(vec![pon]))];
        }
    }
    vec![]
}

fn check_minkan(stg: &Stage) -> Vec<(Seat, PlayerOperation)> {
    if stg.left_tile_count == 0 || stg.doras.len() == 5 {
        return vec![];
    }

    let mut d = stg.players[stg.turn].discards.last().unwrap().tile;
    if d.1 == 0 {
        d.1 = 5; // 赤５対策
    }
    let mut count = [0; SEAT];
    for &e in &stg.tile_states[d.0][d.1] {
        if let H(s) = e {
            count[s] += 1;
        }
    }
    for s in 0..SEAT {
        if count[s] != 3 || stg.turn == s || stg.players[s].is_riichi {
            continue;
        }

        return vec![(s, Minkan(vec![d]))];
    }
    vec![]
}

fn check_ron(stg: &Stage) -> Vec<(Seat, PlayerOperation)> {
    let mut v = vec![];
    for s in 0..SEAT {
        if !stg.players[s].is_furiten && !stg.players[s].is_furiten_other {
            if let Some(_) = evaluate_hand_ron(stg, &vec![], s) {
                v.push((s, Ron));
            }
        }
    }
    v
}

// Utility ====================================================================

fn create_wall(seed: u64) -> Vec<Tile> {
    let mut wall = Vec::new();
    for t in 0..TYPE {
        for i in 1..TNUM {
            if t == 3 && i == 8 {
                break;
            }
            for n in 0..TILE {
                let i2 = if t != TZ && i == 5 && n == 0 { 0 } else { i }; // 赤5
                wall.push(Tile(t, i2));
            }
        }
    }

    let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(seed);
    wall.shuffle(&mut rng);

    return wall;
}

fn calc_prohibited_discards(op: &Option<PlayerOperation>) -> Vec<Tile> {
    let mut v = vec![];
    match op {
        Some(Chii(v2)) => {
            // 赤5が混じっている可能性を考慮
            let (t0, t1) = v2[0];
            let ti = t0.0;
            let ni0 = t0.n();
            let ni1 = t1.n();
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
        Some(Pon(v2)) => {
            let (t, _) = v2[0];
            v.push(Tile(t.0, t.n()));
        }
        _ => return vec![],
    }

    let mut has5 = false;
    for t in &v {
        if t.0 != TZ && t.1 == 5 {
            has5 = true;
        }
    }
    if has5 {
        v.push(Tile(v[0].0, 0));
    }

    v
}

// Application ================================================================

pub struct App {
    seed: u64,
    n_game: i32,
    n_thread: i32,
    debug: bool,
}

impl App {
    pub fn new(args: Vec<String>) -> Self {
        use std::process::exit;

        let mut seed = 0;
        let mut n_game = 0;
        let mut n_thread = 16;
        let mut debug = false;
        let mut it = args.iter();
        while let Some(s) = it.next() {
            match s.as_str() {
                "-s" => {
                    // seed
                    if let Some(n) = it.next() {
                        seed = n.parse().unwrap();
                    } else {
                        println!("-s: Seed missing");
                        exit(0);
                    }
                }
                "-g" => {
                    // game
                    if let Some(n) = it.next() {
                        n_game = n.parse().unwrap();
                    } else {
                        println!("-g: n_game missing");
                    }
                }
                "-t" => {
                    // thread
                    if let Some(n) = it.next() {
                        n_thread = n.parse().unwrap();
                    } else {
                        println!("-t: n_thread missing");
                    }
                }
                "-d" => {
                    // debug
                    debug = true;
                }
                opt => {
                    println!("Unknown option: {}", opt);
                    exit(0);
                }
            }
        }

        Self {
            seed,
            n_game,
            n_thread,
            debug,
        }
    }

    pub fn run(&mut self) {
        if self.seed == 0 {
            self.seed = unixtime_now();
            println!(
                "Random seed is not specified. Unix timestamp '{}' is used as seed.",
                self.seed
            );
        }

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
        use crate::operator::bot2::Bot2;
        use crate::operator::bot_tiitoitsu::TiitoitsuBot; // 七対子bot
                                                          // use crate::operator::manual::ManualOperator;
                                                          // use crate::operator::random::RandomDiscardOperator;

        let config = Config {
            seed: self.seed,
            n_round: 2,
            initial_score: 25000,
            operators: [
                // Box::new(ManualOperator::new()),
                // Box::new(RandomDiscardOperator::new(self.seed + 0)),
                Box::new(Bot2::new()),
                Box::new(TiitoitsuBot::new()),
                Box::new(TiitoitsuBot::new()),
                Box::new(TiitoitsuBot::new()),
            ],
            listeners: vec![Box::new(StageConsolePrinter {})],
        };

        let mut game = MahjongEngine::new(config);
        let send_recv = create_ws_server(52001);

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
                    "data": &game.stage,
                });
                s.send(value.to_string()).ok();
            }

            if self.debug && stop {
                use std::io::{stdin, stdout, Write};
                print!("step={} Enter>", game.stage.step);
                stdout().flush().unwrap();
                let mut buf = String::new();
                stdin().read_line(&mut buf).ok();
            }

            if end {
                break;
            }
        }
    }

    fn run_multiple_game(&mut self) {
        use crate::operator::bot_tiitoitsu::TiitoitsuBot; // 七対子bot
        use crate::operator::random::RandomDiscardOperator;

        use std::sync::mpsc;
        use std::{thread, time};

        let mut n_game = 0;
        let mut n_thread = 0;
        let mut n_game_end = 0;
        let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(self.seed);
        let (tx, rx) = mpsc::channel();
        let operators: [Box<dyn Operator>; 4] = [
            Box::new(TiitoitsuBot::new()),
            Box::new(TiitoitsuBot::new()),
            Box::new(TiitoitsuBot::new()),
            Box::new(RandomDiscardOperator::new(0)),
        ];

        let mut total_score_delta = [0; SEAT];
        let mut total_rank_sum = [0; SEAT];
        loop {
            if n_game < self.n_game && n_thread < self.n_thread {
                n_game += 1;
                n_thread += 1;

                let seed = rng.next_u64();
                let mut shuffle_table = [0, 1, 2, 3];
                shuffle_table.shuffle(&mut rng);

                let mut shuffled_operators: [Box<dyn Operator>; 4] = [
                    Box::new(NullOperator::new()),
                    Box::new(NullOperator::new()),
                    Box::new(NullOperator::new()),
                    Box::new(NullOperator::new()),
                ];
                for s in 0..SEAT {
                    shuffled_operators[s] = operators[shuffle_table[s]].clone_box();
                }

                let tx2 = tx.clone();
                thread::spawn(move || {
                    let config = Config {
                        seed: seed,
                        n_round: 2,
                        initial_score: 25000,
                        operators: shuffled_operators,
                        listeners: vec![],
                    };
                    let start = time::Instant::now();

                    let mut game = MahjongEngine::new(config);
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
                    print!("{:5},{:4}ms,{:20}", n_game_end, ms, game.config.seed);
                    for s in 0..SEAT {
                        let pl = &game.stage.players[s];
                        let (score, rank) = (pl.score, pl.rank + 1);
                        let i = shuffle[s];
                        total_score_delta[i] += score - game.config.initial_score;
                        total_rank_sum[i] += rank;
                        print!(", op{}:{:5}({})", i, score, rank);
                    }
                    println!();

                    n_thread -= 1;
                    n_game_end += 1;
                }
                if n_thread < self.n_thread {
                    break;
                }
                thread::sleep(time::Duration::from_millis(10));
            }

            if n_thread == 0 && n_game == self.n_game {
                for i in 0..SEAT {
                    println!(
                        "op{} avg_rank: {:.2}, avg_score_delta: {:6}",
                        i,
                        total_rank_sum[i] as f32 / n_game as f32,
                        total_score_delta[i] / n_game,
                    );
                }
                break;
            }
        }
    }
}
