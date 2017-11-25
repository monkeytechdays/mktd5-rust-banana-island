use super::model::{Game, Move};

pub type AI = StandStill;

pub struct StandStill {
}
impl StandStill {
    pub fn new() -> StandStill {
        StandStill {
        }
    }

    pub fn play(&mut self, _game: &mut Game) -> Move {
        Move::None
    }
}
