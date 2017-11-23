use super::model::{GameState, Move};

pub type AI = StandStill;

pub struct StandStill {
}
impl StandStill {
    pub fn new() -> StandStill {
        StandStill {
        }
    }

    pub fn play(&mut self, _game: &mut GameState) -> Move {
        Move::None
    }
}
