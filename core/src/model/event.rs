use crate::hand::WinContext;
use crate::model::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    GameStart(EventGameStart),
    RoundNew(EventRoundNew),
    DealTile(EventDealTile),
    DiscardTile(EventDiscardTile),
    Meld(EventMeld),
    Kita(EventKita),
    Dora(EventDora),
    RoundEndWin(EventRoundEndWin),
    RoundEndDraw(EventRoundEndDraw),
    RoundEndNoTile(EventRoundEndNoTile),
    GameOver(EventGameOver),
}

impl Event {
    pub fn game_start() -> Self {
        Self::GameStart(EventGameStart {})
    }

    pub fn round_new(
        round: usize,
        kyoku: usize,
        honba: usize,
        kyoutaku: usize,
        doras: Vec<Tile>,
        scores: [Score; SEAT],
        hands: [Vec<Tile>; SEAT],
        mode: usize,
    ) -> Self {
        Self::RoundNew(EventRoundNew {
            round,
            kyoku,
            honba,
            kyoutaku,
            doras,
            scores,
            hands,
            mode,
        })
    }

    pub fn deal_tile(seat: Seat, tile: Tile) -> Self {
        Self::DealTile(EventDealTile { seat, tile })
    }

    pub fn discard_tile(seat: Seat, tile: Tile, is_drawn: bool, is_riichi: bool) -> Self {
        Self::DiscardTile(EventDiscardTile {
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

    pub fn round_end_win(
        ura_doras: Vec<Tile>,
        contexts: Vec<(Seat, [Point; SEAT], WinContext)>,
    ) -> Self {
        Self::RoundEndWin(EventRoundEndWin {
            ura_doras,
            contexts,
        })
    }

    pub fn round_end_draw(draw_type: DrawType) -> Self {
        Self::RoundEndDraw(EventRoundEndDraw { draw_type })
    }

    pub fn round_end_no_tile(tenpais: [bool; SEAT], points: [Point; SEAT]) -> Self {
        Self::RoundEndNoTile(EventRoundEndNoTile { tenpais, points })
    }

    pub fn game_over() -> Self {
        Self::GameOver(EventGameOver {})
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventGameStart {}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventRoundNew {
    pub round: usize,
    pub kyoku: usize,
    pub honba: usize,
    pub kyoutaku: usize,
    pub doras: Vec<Tile>,
    pub scores: [Score; SEAT],
    pub hands: [Vec<Tile>; SEAT],
    pub mode: usize, // 1: 4人東, 2: 4人南
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventDealTile {
    pub seat: Seat,
    pub tile: Tile,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventDiscardTile {
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
pub struct EventRoundEndWin {
    pub ura_doras: Vec<Tile>,
    pub contexts: Vec<(Seat, [Point; SEAT], WinContext)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventRoundEndDraw {
    pub draw_type: DrawType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventRoundEndNoTile {
    pub tenpais: [bool; SEAT],
    pub points: [Point; SEAT],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventGameOver {}
