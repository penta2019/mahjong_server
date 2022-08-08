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
    pub fn begin() -> Self {
        Self::Begin(EventBegin {})
    }

    pub fn new(
        bakaze: usize,
        kyoku: usize,
        honba: usize,
        kyoutaku: usize,
        doras: Vec<Tile>,
        scores: [Score; SEAT],
        hands: [Vec<Tile>; SEAT],
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
            mode,
        })
    }

    pub fn deal(seat: Seat, tile: Tile) -> Self {
        Self::Deal(EventDeal { seat, tile })
    }

    pub fn discard(seat: Seat, tile: Tile, is_drawn: bool, is_riichi: bool) -> Self {
        Self::Discard(EventDiscard {
            seat,
            tile,
            is_drawn,
            is_riichi,
        })
    }

    pub fn meld(seat: Seat, meld_type: MeldType, consumed: Vec<Tile>) -> Self {
        Self::Meld(EventMeld {
            seat,
            meld_type,
            consumed,
        })
    }

    pub fn kita(seat: Seat, is_drawn: bool) -> Self {
        Self::Kita(EventKita { seat, is_drawn })
    }

    pub fn dora(tile: Tile) -> Self {
        Self::Dora(EventDora { tile })
    }

    pub fn win(ura_doras: Vec<Tile>, contexts: Vec<(Seat, [Point; SEAT], WinContext)>) -> Self {
        Self::Win(EventWin {
            ura_doras,
            contexts,
        })
    }

    pub fn draw(
        type_: DrawType,
        hands: [Vec<Tile>; SEAT],
        tenpais: [bool; SEAT],
        points: [Point; SEAT],
    ) -> Self {
        Self::Draw(EventDraw {
            type_,
            hands,
            tenpais,
            points,
        })
    }

    pub fn end() -> Self {
        Self::End(EventEnd {})
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventBegin {}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventNew {
    pub bakaze: usize,
    pub kyoku: usize,
    pub honba: usize,
    pub kyoutaku: usize,
    pub doras: Vec<Tile>,
    pub scores: [Score; SEAT],
    pub hands: [Vec<Tile>; SEAT],
    pub mode: usize, // 1: 4人東, 2: 4人南
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventDeal {
    pub seat: Seat,
    pub tile: Tile,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventDiscard {
    pub seat: Seat,
    pub tile: Tile,
    pub is_drawn: bool,
    pub is_riichi: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventMeld {
    pub seat: Seat,
    pub meld_type: MeldType,
    pub consumed: Vec<Tile>,
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
    pub ura_doras: Vec<Tile>,
    pub contexts: Vec<(Seat, [Point; SEAT], WinContext)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventDraw {
    pub type_: DrawType,
    pub hands: [Vec<Tile>; SEAT],
    pub tenpais: [bool; SEAT],
    pub points: [Point; SEAT],
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
