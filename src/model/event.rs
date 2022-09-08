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
        bakaze: usize,
        kyoku: usize,
        honba: usize,
        kyoutaku: usize,
        doras: Vec<Tile>,
        scores: [Score; SEAT],
        hands: [Vec<Tile>; SEAT],
        wall_count: usize,
        mode: usize,
    ) -> Self {
        Self::New(EventNew {
            bakaze,
            kyoku,
            honba,
            kyoutaku,
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
    pub fn win(ura_doras: Vec<Tile>, contexts: Vec<(Seat, [Point; SEAT], WinContext)>) -> Self {
        Self::Win(EventWin {
            ura_doras,
            contexts,
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
    pub bakaze: usize,            // 場風
    pub kyoku: usize,             // 局
    pub honba: usize,             // 本場
    pub kyoutaku: usize,          // 供託(リーチ棒)
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
    pub ura_doras: Vec<Tile>, // 裏ドラ表示牌
    pub contexts: Vec<(Seat, [Point; SEAT], WinContext)>,
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
