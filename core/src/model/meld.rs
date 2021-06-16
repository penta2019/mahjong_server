use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MeldType {
    Chi,
    Pon,
    Minkan,
    Kakan,
    Ankan,
}

#[derive(Debug, Serialize)]
pub struct Meld {
    pub step: usize,
    pub seat: Seat,
    pub type_: MeldType,
    pub tiles: Vec<Tile>,
    pub froms: Vec<Seat>,
}

impl fmt::Display for Meld {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let z = self.tiles.iter().zip(self.froms.iter());
        let s: Vec<String> = z.map(|x| format!("{}({})", x.0, x.1)).collect();
        write!(f, "{}", s.join("|"))
    }
}
