use super::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    Begin(EventBegin),     // ゲーム開始
    New(EventNew),         // 局開始
    Deal(EventDeal),       // ツモ
    Discard(EventDiscard), // 打牌
    Meld(EventMeld),       // 鳴き
    Nukidora(EventKita),   // 北抜き
    Dora(EventDora),       // 新ドラ
    Win(EventWin),         // 局終了 (和了)
    Draw(EventDraw),       // 局終了 (流局)
    End(EventEnd),         // ゲーム終了
}

impl Event {
    #[inline]
    pub fn begin() -> Self {
        Self::Begin(EventBegin {})
    }

    #[inline]
    pub fn new(
        rule: Rule,
        round: usize,
        dealer: usize,
        honba_sticks: usize,
        riichi_sticks: usize,
        doras: Vec<Tile>,
        names: [String; SEAT],
        scores: [Score; SEAT],
        hands: [Vec<Tile>; SEAT],
        wall_count: usize,
    ) -> Self {
        Self::New(EventNew {
            rule,
            round,
            dealer,
            honba_sticks,
            riichi_sticks,
            doras,
            names,
            scores,
            hands,
            wall_count,
        })
    }

    #[inline]
    pub fn deal(seat: Seat, tile: Tile) -> Self {
        Self::Deal(EventDeal { seat, tile })
    }

    #[inline]
    pub fn discard(seat: Seat, tile: Tile, is_drop: bool, is_riichi: bool) -> Self {
        Self::Discard(EventDiscard {
            seat,
            tile,
            is_drop,
            is_riichi,
        })
    }

    #[inline]
    pub fn meld(seat: Seat, meld_type: MeldType, consumed: Vec<Tile>, is_pao: bool) -> Self {
        Self::Meld(EventMeld {
            seat,
            meld_type,
            consumed,
            is_pao,
        })
    }

    #[inline]
    pub fn nukidora(seat: Seat, is_drop: bool) -> Self {
        Self::Nukidora(EventKita { seat, is_drop })
    }

    #[inline]
    pub fn dora(tile: Tile) -> Self {
        Self::Dora(EventDora { tile })
    }

    #[inline]
    pub fn win(
        round: usize,
        dealer: usize,
        honba_sticks: usize,
        riichi_sticks: usize,
        doras: Vec<Tile>,
        ura_doras: Vec<Tile>,
        names: [String; SEAT],
        scores: [Point; SEAT],
        delta_scores: [Point; SEAT],
        contexts: Vec<WinContext>,
    ) -> Self {
        Self::Win(EventWin {
            round,
            dealer,
            honba_sticks,
            riichi_sticks,
            doras,
            ura_doras,
            names,
            scores,
            delta_scores,
            contexts,
        })
    }

    #[inline]
    pub fn draw(
        draw_type: DrawType,
        round: usize,
        dealer: usize,
        names: [String; SEAT],
        scores: [Point; SEAT],
        delta_scores: [Point; SEAT],
        nagashimangan_scores: [Point; SEAT],
        hands: [Vec<Tile>; SEAT],
    ) -> Self {
        Self::Draw(EventDraw {
            draw_type,
            round,
            dealer,
            names,
            scores,
            delta_scores,
            nagashimangan_scores,
            hands,
        })
    }

    #[inline]
    pub fn end() -> Self {
        Self::End(EventEnd {})
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventBegin {}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventNew {
    pub rule: Rule,               // ゲーム設定
    pub round: usize,             // 場風
    pub dealer: usize,            // 局
    pub honba_sticks: usize,      // 本場
    pub riichi_sticks: usize,     // 供託(リーチ棒)
    pub doras: Vec<Tile>,         // ドラ表示牌
    pub names: [String; SEAT],    // プレイヤー名
    pub scores: [Score; SEAT],    // 各プレイヤーの所持点
    pub hands: [Vec<Tile>; SEAT], // 各プレイヤーの手牌(親:14枚, 子:13枚)
    pub wall_count: usize,        // 牌山残り枚数
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventDeal {
    pub seat: Seat,
    pub tile: Tile, // ツモ牌
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventDiscard {
    pub seat: Seat,
    pub tile: Tile,
    pub is_drop: bool,   // ツモ切り
    pub is_riichi: bool, // リーチ宣言
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventMeld {
    pub seat: Seat,
    pub meld_type: MeldType, // 鳴き種別
    pub consumed: Vec<Tile>, // 手牌から消費される牌
    pub is_pao: bool,        // 責任払いが発生するかどうか
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventKita {
    pub seat: Seat,
    pub is_drop: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventDora {
    pub tile: Tile,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventWin {
    // Stageを見れば明らかにわかることであっても和了結果表示に必要な情報はすべてここに含める
    // StageControllerにとって必要なデータはdelta_scoresのみ
    pub round: usize,
    pub dealer: usize,
    pub honba_sticks: usize,
    pub riichi_sticks: usize,
    pub doras: Vec<Tile>,            // ドラ表示牌
    pub ura_doras: Vec<Tile>,        // 裏ドラ表示牌
    pub names: [String; SEAT],       // プレイヤー名
    pub scores: [Point; SEAT],       // 変化前のスコア
    pub delta_scores: [Point; SEAT], // scores + delta_scores = new_scores
    pub contexts: Vec<WinContext>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventDraw {
    // EventWin同様に流局表示に必要な情報をすべて含める
    pub draw_type: DrawType,
    pub round: usize,
    pub dealer: usize,
    pub names: [String; SEAT],               // プレイヤー名
    pub scores: [Point; SEAT],               // 変化前のスコア
    pub delta_scores: [Point; SEAT],         // 聴牌,流し満貫による点数変動
    pub nagashimangan_scores: [Point; SEAT], // 流し満貫のスコア (該当者がいない場合すべて0)
    pub hands: [Vec<Tile>; SEAT],            // 聴牌していたプレイヤーの手牌 (ノーテンは空の配列)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventEnd {}

// [DrawType]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrawType {
    Unknown,        // 不明
    Kyushukyuhai,   // 九種九牌
    Suufuurenda,    // 四風連打
    Suukansanra,    // 四槓散了
    Suuchariichi,   // 四家立直
    Sanchaho,       // 三家和
    Kouhaiheikyoku, // 荒廃平局
}

impl fmt::Display for DrawType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}",
            match self {
                DrawType::Unknown => "不明",
                DrawType::Kyushukyuhai => "九種九牌",
                DrawType::Suufuurenda => "四風連打",
                DrawType::Suukansanra => "四槓散了",
                DrawType::Suuchariichi => "四家立直",
                DrawType::Sanchaho => "三家和",
                DrawType::Kouhaiheikyoku => "荒廃平局",
            }
        )
    }
}
