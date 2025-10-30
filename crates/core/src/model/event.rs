use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    Begin(EventBegin),       // ゲーム開始
    New(EventNew),           // 局開始
    Deal(EventDeal),         // ツモ
    Discard(EventDiscard),   // 打牌
    Meld(EventMeld),         // 鳴き
    Nukidora(EventNukidora), // 北抜き
    Dora(EventDora),         // 新ドラ
    Win(EventWin),           // 局終了 (和了)
    Draw(EventDraw),         // 局終了 (流局)
    End(EventEnd),           // ゲーム終了
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
        dealer: Seat,
        honba: usize,
        riichi_sticks: usize,
        doras: Vec<Tile>,
        names: [String; SEAT],
        scores: [Score; SEAT],
        hands: [Vec<Tile>; SEAT],
        wall_count: usize,
        dice: usize,
        wall: Vec<Tile>,
        dora_wall: Vec<Tile>,
        ura_dora_wall: Vec<Tile>,
        replacement_wall: Vec<Tile>,
    ) -> Self {
        Self::New(EventNew {
            rule,
            round,
            dealer,
            honba,
            riichi_sticks,
            doras,
            names,
            scores,
            hands,
            wall_count,
            dice,
            wall,
            dora_wall,
            ura_dora_wall,
            replacement_wall,
        })
    }

    #[inline]
    pub fn deal(seat: Seat, tile: Tile, is_replacement: bool) -> Self {
        Self::Deal(EventDeal {
            seat,
            tile,
            is_replacement,
        })
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
    pub fn meld(seat: Seat, meld_type: MeldType, consumed: Vec<Tile>, is_pao: bool) -> Self {
        Self::Meld(EventMeld {
            seat,
            meld_type,
            consumed,
            is_pao,
        })
    }

    #[inline]
    pub fn nukidora(seat: Seat, is_drawn: bool) -> Self {
        Self::Nukidora(EventNukidora { seat, is_drawn })
    }

    #[inline]
    pub fn dora(tile: Tile) -> Self {
        Self::Dora(EventDora { tile })
    }

    #[inline]
    pub fn win(
        round: usize,
        dealer: Seat,
        honba: usize,
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
            honba,
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
        dealer: Seat,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBegin {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventNew {
    pub rule: Rule,                  // ゲーム設定
    pub round: usize,                // 場風
    pub dealer: Seat,                // 局
    pub honba: usize,                // 本場
    pub riichi_sticks: usize,        // 供託(リーチ棒)
    pub doras: Vec<Tile>,            // ドラ表示牌
    pub names: [String; SEAT],       // プレイヤー名
    pub scores: [Score; SEAT],       // 各プレイヤーの所持点
    pub hands: [Vec<Tile>; SEAT],    // 各プレイヤーの手牌(13枚 親の14枚目も通常のツモとして扱う)
    pub wall_count: usize,           // 牌山残り枚数
    pub dice: usize,                 // サイコロの目の和
    pub wall: Vec<Tile>,             // 牌譜用 牌山
    pub dora_wall: Vec<Tile>,        // 牌譜用 ドラ表示牌 (5枚)
    pub ura_dora_wall: Vec<Tile>,    // 牌譜用 裏ドラ (5枚)
    pub replacement_wall: Vec<Tile>, // 牌譜用 嶺上牌 (4枚)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDeal {
    pub seat: Seat,
    pub tile: Tile,           // ツモ牌
    pub is_replacement: bool, // 嶺上牌の場合true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDiscard {
    pub seat: Seat,
    pub tile: Tile,
    pub is_drawn: bool,  // ツモ切り
    pub is_riichi: bool, // リーチ宣言
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMeld {
    pub seat: Seat,
    pub meld_type: MeldType, // 鳴き種別
    pub consumed: Vec<Tile>, // 手牌から消費される牌
    pub is_pao: bool,        // 責任払いが発生するかどうか
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventNukidora {
    pub seat: Seat,
    pub is_drawn: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDora {
    pub tile: Tile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventWin {
    // Stageを見れば明らかにわかることであっても和了結果表示に必要な情報はすべてここに含める
    // StageControllerにとって必要なデータはdelta_scoresのみ
    pub round: usize,
    pub dealer: Seat,
    pub honba: usize,
    pub riichi_sticks: usize,
    pub doras: Vec<Tile>,            // ドラ表示牌
    pub ura_doras: Vec<Tile>,        // 裏ドラ表示牌
    pub names: [String; SEAT],       // プレイヤー名
    pub scores: [Point; SEAT],       // 変化前のスコア
    pub delta_scores: [Point; SEAT], // scores + delta_scores = new_scores
    pub contexts: Vec<WinContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDraw {
    // EventWin同様に流局表示に必要な情報をすべて含める
    pub draw_type: DrawType,
    pub round: usize,
    pub dealer: Seat,
    pub names: [String; SEAT],               // プレイヤー名
    pub scores: [Point; SEAT],               // 変化前のスコア
    pub delta_scores: [Point; SEAT],         // 聴牌,流し満貫による点数変動
    pub nagashimangan_scores: [Point; SEAT], // 流し満貫のスコア (該当者がいない場合すべて0)
    pub hands: [Vec<Tile>; SEAT],            // 聴牌していたプレイヤーの手牌 (ノーテンは空の配列)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
                DrawType::Kouhaiheikyoku => "荒牌平局",
            }
        )
    }
}
