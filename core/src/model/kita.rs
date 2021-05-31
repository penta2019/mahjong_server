use super::*;

#[derive(Debug, Serialize)]
pub struct Kita {
    pub step: usize,
    pub seat: Seat,
    pub drawn: bool,
}

impl fmt::Display for Kita {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "z4")
    }
}
