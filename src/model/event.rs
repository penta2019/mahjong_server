use super::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    Begin(EventBegin),     // ゲーム開始
    New(EventNew),         // 局開始
    Deal(EventDeal),       // ツモ
    Discard(EventDiscard), // 打牌
    Meld(EventMeld),       // 鳴き
    Kita(EventKita),       // 北抜き
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
        round: usize,
        dealer: usize,
        honba_sticks: usize,
        riichi_sticks: usize,
        doras: Vec<Tile>,
        scores: [Score; SEAT],
        hands: [Vec<Tile>; SEAT],
        wall_count: usize,
        mode: usize,
    ) -> Self {
        Self::New(EventNew {
            round,
            dealer,
            honba_sticks,
            riichi_sticks,
            doras,
            scores,
            hands,
            wall_count,
            mode,
        })
    }

    #[inline]
    pub fn deal(seat: Seat, tile: Tile) -> Self {
        Self::Deal(EventDeal { seat, tile })
    }

    #[inline]
    pub fn discard(seat: Seat, tile: Tile, is_drawn: bool, is_riichi: bool) -> Self {
        Self::Discard(EventDiscard {
            seat,
            tile,
            is_drawn,
            is_riichi,
        })
    }

    #[inline]
    pub fn meld(seat: Seat, meld_type: MeldType, consumed: Vec<Tile>) -> Self {
        Self::Meld(EventMeld {
            seat,
            meld_type,
            consumed,
        })
    }

    #[inline]
    pub fn kita(seat: Seat, is_drawn: bool) -> Self {
        Self::Kita(EventKita { seat, is_drawn })
    }

    #[inline]
    pub fn dora(tile: Tile) -> Self {
        Self::Dora(EventDora { tile })
    }

    #[inline]
    pub fn win(
        contexts: Vec<WinContext>,
        doras: Vec<Tile>,
        ura_doras: Vec<Tile>,
        scores: [Point; SEAT],
        delta_scores: [Point; SEAT],
    ) -> Self {
        Self::Win(EventWin {
            contexts,
            doras,
            ura_doras,
            scores,
            delta_scores,
        })
    }

    #[inline]
    pub fn draw(
        draw_type: DrawType,
        hands: [Vec<Tile>; SEAT],
        delta_scores: [Point; SEAT],
        nagashimangan_scores: [Point; SEAT],
    ) -> Self {
        Self::Draw(EventDraw {
            draw_type,
            hands,
            delta_scores,
            nagashimangan_scores,
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
    pub round: usize,             // 場風
    pub dealer: usize,            // 局
    pub honba_sticks: usize,      // 本場
    pub riichi_sticks: usize,     // 供託(リーチ棒)
    pub doras: Vec<Tile>,         // ドラ表示牌
    pub scores: [Score; SEAT],    // 各プレイヤーの所持点
    pub hands: [Vec<Tile>; SEAT], // 各プレイヤーの手牌(親:14枚, 子:13枚)
    pub wall_count: usize,        // 牌山残り枚数
    pub mode: usize,              // 1: 4人東, 2: 4人南
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
    pub is_drawn: bool,  // ツモ切り
    pub is_riichi: bool, // リーチ宣言
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventMeld {
    pub seat: Seat,
    pub meld_type: MeldType, // 鳴き種別
    pub consumed: Vec<Tile>, // 手牌から消費される牌
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventKita {
    pub seat: Seat,
    pub is_drawn: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventDora {
    pub tile: Tile,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventWin {
    // Stageを見れば明らかにわかることであっても和了演出に必要な情報はすべてここに含める
    // StageControllerにとって必要なデータはdelta_scoresのみ
    pub contexts: Vec<WinContext>,
    pub doras: Vec<Tile>,            // ドラ表示牌
    pub ura_doras: Vec<Tile>,        // 裏ドラ表示牌
    pub scores: [Point; SEAT],       // 変化前のスコア
    pub delta_scores: [Point; SEAT], // scores + delta_scores = new_scores
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventDraw {
    pub draw_type: DrawType,
    pub hands: [Vec<Tile>; SEAT], // 聴牌していたプレイヤーの手牌 (ノーテンは空の配列)
    pub delta_scores: [Point; SEAT], // 聴牌,流し満貫による点数変動
    pub nagashimangan_scores: [Point; SEAT], // 流し満貫のスコア (該当者がいない場合すべて0)
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
