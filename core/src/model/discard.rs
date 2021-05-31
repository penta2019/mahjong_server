use super::*;

#[derive(Debug, Serialize)]
pub struct Discard {
    pub step: usize,
    pub tile: Tile,
    pub drawn: bool,                 // ツモ切りフラグ
    pub meld: Option<(Seat, Index)>, // 鳴きが入った場合にセット
}

impl fmt::Display for Discard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.tile.to_string())
    }
}
