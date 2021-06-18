use crate::hand::evaluate::WinContext;
use crate::model::*;

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum Action {
    GameStart(ActionGameStart),
    RoundNew(ActionRoundNew),
    DealTile(ActionDealTile),
    DiscardTile(ActionDiscardTile),
    Meld(ActionMeld),
    Kita(ActionKita),
    Dora(ActionDora),
    RoundEndWin(ActionRoundEndWin),
    RoundEndDraw(ActionRoundEndDraw),
    RoundEndNoTile(ActionRoundEndNoTile),
    GameOver(ActionGameOver),
}

impl Action {
    pub fn game_start() -> Self {
        Self::GameStart(ActionGameStart {})
    }

    pub fn round_new(
        round: usize,
        kyoku: usize,
        honba: usize,
        kyoutaku: usize,
        doras: Vec<Tile>,
        scores: [Score; SEAT],
        hands: [Vec<Tile>; SEAT],
    ) -> Self {
        Self::RoundNew(ActionRoundNew {
            round,
            kyoku,
            honba,
            kyoutaku,
            doras,
            scores,
            hands,
        })
    }

    pub fn deal_tile(seat: Seat, tile: Tile) -> Self {
        Self::DealTile(ActionDealTile { seat, tile })
    }

    pub fn discard_tile(seat: Seat, tile: Tile, is_drawn: bool, is_riichi: bool) -> Self {
        Self::DiscardTile(ActionDiscardTile {
            seat,
            tile,
            is_drawn,
            is_riichi,
        })
    }

    pub fn meld(seat: Seat, meld_type: MeldType, consumed: Vec<Tile>) -> Self {
        Self::Meld(ActionMeld {
            seat,
            meld_type,
            consumed,
        })
    }

    pub fn kita(seat: Seat, is_drawn: bool) -> Self {
        Self::Kita(ActionKita { seat, is_drawn })
    }

    pub fn dora(tile: Tile) -> Self {
        Self::Dora(ActionDora { tile })
    }

    pub fn round_end_win(
        ura_doras: Vec<Tile>,
        contexts: Vec<(Seat, [Point; SEAT], WinContext)>,
    ) -> Self {
        Self::RoundEndWin(ActionRoundEndWin {
            ura_doras,
            contexts,
        })
    }

    pub fn round_end_draw(draw_type: DrawType) -> Self {
        Self::RoundEndDraw(ActionRoundEndDraw { draw_type })
    }

    pub fn round_end_no_tile(tenpais: [bool; SEAT], points: [Point; SEAT]) -> Self {
        Self::RoundEndNoTile(ActionRoundEndNoTile { tenpais, points })
    }

    pub fn game_over() -> Self {
        Self::GameOver(ActionGameOver {})
    }
}

#[derive(Debug, Serialize)]
pub struct ActionGameStart {}

#[derive(Debug, Serialize)]
pub struct ActionRoundNew {
    pub round: usize,
    pub kyoku: usize,
    pub honba: usize,
    pub kyoutaku: usize,
    pub doras: Vec<Tile>,
    pub scores: [Score; SEAT],
    pub hands: [Vec<Tile>; SEAT],
}

#[derive(Debug, Serialize)]
pub struct ActionDealTile {
    pub seat: Seat,
    pub tile: Tile,
}

#[derive(Debug, Serialize)]
pub struct ActionDiscardTile {
    pub seat: Seat,
    pub tile: Tile,
    pub is_drawn: bool,
    pub is_riichi: bool,
}

#[derive(Debug, Serialize)]
pub struct ActionMeld {
    pub seat: Seat,
    pub meld_type: MeldType,
    pub consumed: Vec<Tile>,
}

#[derive(Debug, Serialize)]
pub struct ActionKita {
    pub seat: Seat,
    pub is_drawn: bool,
}

#[derive(Debug, Serialize)]
pub struct ActionDora {
    pub tile: Tile,
}

#[derive(Debug, Serialize)]
pub struct ActionRoundEndWin {
    pub ura_doras: Vec<Tile>,
    pub contexts: Vec<(Seat, [Point; SEAT], WinContext)>,
}

#[derive(Debug, Serialize)]
pub struct ActionRoundEndDraw {
    pub draw_type: DrawType,
}

#[derive(Debug, Serialize)]
pub struct ActionRoundEndNoTile {
    pub tenpais: [bool; SEAT],
    pub points: [Point; SEAT],
}

#[derive(Debug, Serialize)]
pub struct ActionGameOver {}
