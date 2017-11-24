use std::sync::{Arc, Mutex};
use serde::{Deserialize, Deserializer};

#[derive(Default, Debug)]
pub struct GameState {
    player_id: Option<i64>,
    map: Option<Map>,
}
impl GameState {
    pub fn player_mut(&mut self) -> &mut Option<i64> {
        &mut self.player_id
    }

    pub fn player(&self) -> i64 {
        self.player_id.unwrap()
    }

    pub fn set_map(&mut self, map: Map) {
        self.map = Some(map);
    }

    pub fn map_mut(&mut self) -> &mut Map {
        (&mut self.map).as_mut().unwrap()
    }
}

pub type GameStateMutex = Arc<Mutex<GameState>>;


#[derive(Deserialize, Debug)]
pub struct Map {
    #[serde(rename="map")]
    pub content: Vec<Vec<Entity>>,
}
impl Map {
    pub fn apply_move(&mut self, player_move: &PlayerMove) {
        if player_move.direction == Move::None {
            return;
        }
        let player = Entity::Player(player_move.id);

        let mut indexes: Option<((usize, usize),(usize, usize))> = None;
        'global: for row_index in 0..self.content.len() {
            let row = &self.content[row_index];
            for col_index in 0..row.len() {
                let val = row[col_index];
                if val == player {
                    if let Some((target_row_index, target_col_index)) = player_move.direction.move_from(row_index, col_index) {

                        if target_row_index < self.content.len() {
                            let target_row = &self.content[target_row_index];
                            if target_col_index < target_row.len() {
                                indexes = Some(((row_index, col_index), (target_row_index, target_col_index)));
                            }
                        }
                    }

                    break 'global;
                }
            }
        }

        if let Some(((src_row, src_col),(tgt_row, tgt_col))) = indexes {
            self.content[src_row][src_col] = Entity::Empty;
            self.content[tgt_row][tgt_col] = player;
        }
    }
}

#[cfg(test)]
mod map_should {

    use super::{Map, PlayerMove, Move, Entity};
    use super::Entity::*;
    use serde_json;

    fn content(matrix: [[Entity; 3]; 3]) -> Vec<Vec<Entity>> {
        matrix.into_iter()
               .map(|row| row.to_vec())
               .collect()
    }

    fn assert_content(expected: [[Entity; 3]; 3], map: Map) {
        assert_eq!(content(expected), map.content);
    }

    fn new_map(matrix: [[Entity; 3]; 3]) -> Map {
        let content = content(matrix);
        Map { content }
    }

    #[test]
    fn deserialize() {
        let data = r#"{
            "map": [
                [1, 2, 3],
                [4, 5, 6],
                [7, 8, 9]
            ]
        }"#;
        let map: super::Map = serde_json::from_str(data).expect("Can't parse map resource");
        assert_content([[Banana, Wall, Player(3)], [Player(4), Player(5), Player(6)], [Player(7), Player(8), Player(9)]], map);
    }

    #[test]
    fn do_nothing_when_applying_move_None() {
        let mut map = new_map([
            [Empty, Empty    , Empty],
            [Empty, Player(3), Empty],
            [Empty, Empty    , Empty]
        ]);
        map.apply_move(&PlayerMove { id: 3, direction: Move::None });
        assert_content(
            [
                [Empty, Empty    , Empty],
                [Empty, Player(3), Empty],
                [Empty, Empty    , Empty]
            ], map);
    }

    #[test]
    fn move_up_when_applying_move_North() {
        let mut map = new_map([[Empty, Empty, Empty], [Empty, Player(3), Empty], [Empty, Empty, Empty]]);
        map.apply_move(&PlayerMove { id: 3, direction: Move::North });
        assert_content([[Empty, Player(3), Empty], [Empty, Empty, Empty], [Empty, Empty, Empty]], map);
    }

    #[test]
    fn move_left_when_applying_move_West() {
        let mut map = new_map([[Empty, Empty, Empty], [Empty, Player(3), Empty], [Empty, Empty, Empty]]);
        map.apply_move(&PlayerMove { id: 3, direction: Move::West });
        assert_content([[Empty, Empty, Empty], [Player(3), Empty, Empty], [Empty, Empty, Empty]], map);
    }

    #[test]
    fn move_down_when_applying_move_South() {
        let mut map = new_map([[Empty, Empty, Empty], [Empty, Player(3), Empty], [Empty, Empty, Empty]]);
        map.apply_move(&PlayerMove { id: 3, direction: Move::South });
        assert_content([[Empty, Empty, Empty], [Empty, Empty, Empty], [Empty, Player(3), Empty]], map);
    }

    #[test]
    fn move_right_when_applying_move_East() {
        let mut map = new_map([[Empty, Empty, Empty], [Empty, Player(3), Empty], [Empty, Empty, Empty]]);
        map.apply_move(&PlayerMove { id: 3, direction: Move::East });
        assert_content([[Empty, Empty, Empty], [Empty, Empty, Player(3)], [Empty, Empty, Empty]], map);
    }

    #[test]
    fn do_nothing_when_applying_move_North_on_north_border() {
        let mut map = new_map([[Empty, Player(3), Empty], [Empty, Empty, Empty], [Empty, Empty, Empty]]);
        map.apply_move(&PlayerMove { id: 3, direction: Move::North });
        assert_content([[Empty, Player(3), Empty], [Empty, Empty, Empty], [Empty, Empty, Empty]], map);
    }

    #[test]
    fn do_nothing_when_applying_move_West_on_west_border() {
        let mut map = new_map([[Empty, Empty, Empty], [Player(3), Empty, Empty], [Empty, Empty, Empty]]);
        map.apply_move(&PlayerMove { id: 3, direction: Move::West });
        assert_content([[Empty, Empty, Empty], [Player(3), Empty, Empty], [Empty, Empty, Empty]], map);
    }

    #[test]
    fn do_nothing_when_applying_move_South_on_south_border() {
        let mut map = new_map([[Empty, Empty, Empty], [Empty, Empty, Empty], [Empty, Player(3), Empty]]);
        map.apply_move(&PlayerMove { id: 3, direction: Move::South });
        assert_content([[Empty, Empty, Empty], [Empty, Empty, Empty], [Empty, Player(3), Empty]], map);
    }

    #[test]
    fn do_nothing_when_applying_move_East_on_east_border() {
        let mut map = new_map([[Empty, Empty, Empty], [Empty, Empty, Player(3)], [Empty, Empty, Empty]]);
        map.apply_move(&PlayerMove { id: 3, direction: Move::East });
        assert_content([[Empty, Empty, Empty], [Empty, Empty, Player(3)], [Empty, Empty, Empty]], map);
    }
}


#[derive(Deserialize, Serialize, PartialEq, Debug, Clone, Copy)]
pub enum Move {
    #[serde(rename="N")]
    North,
    #[serde(rename="W")]
    West,
    #[serde(rename="S")]
    South,
    #[serde(rename="E")]
    East,
    #[serde(rename="O")]
    None,
}
impl Move {
    fn move_from(&self, row: usize, col: usize) -> Option<(usize, usize)> {
        Some(match *self {
            Move::North if row > 0  => (row-1, col  ),
            Move::West  if col > 0  => (row  , col-1),
            Move::South             => (row+1, col  ),
            Move::East              => (row  , col+1),
            _                       => return None,
        })
    }
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct PlayerMove {
    pub id: i64,
    #[serde(rename = "move")]
    pub direction: Move,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct GameTick {
    id: i64,
    moves: Option<Vec<PlayerMove>>,
}
impl GameTick {
    pub fn moves(&self) -> &Option<Vec<PlayerMove>> {
        &self.moves
    }
}

#[cfg(test)]
mod game_tick_should {

    use super::{GameTick, PlayerMove, Move};

    fn parse(data: &str) -> GameTick {
        ::serde_json::from_str(data).expect("Can't parse data")
    }

    #[test]
    fn deserialize_with_only_id() {
        let data = r#"{
            "id": 42
        }"#;
        assert_eq!(GameTick { id: 42, moves: None }, parse(data));
    }

    #[test]
    fn deserialize_with_empty_moves() {
        let data = r#"{
            "id": 42,
            "moves": []
        }"#;
        assert_eq!(GameTick { id: 42, moves: Some(vec![]) }, parse(data));
    }

    #[test]
    fn deserialize_with_single_moves() {
        let data = r#"{
            "id": 42,
            "moves": [{"id": 314, "move": "O"}]
        }"#;
        assert_eq!(GameTick { id: 42, moves: Some(vec![PlayerMove { id: 314, direction: Move::None}]) }, parse(data));
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Entity {
    Empty,
    Banana,
    Wall,
    Player(i64),
}

impl Entity {
    pub fn from_i64(val: i64) -> Entity {
        match val {
            0  => Entity::Empty,
            1  => Entity::Banana,
            2  => Entity::Wall,
            id => Entity::Player(id),
        }
    }
}

impl<'de> Deserialize<'de> for Entity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let val = i64::deserialize(deserializer)?;
        Ok(Entity::from_i64(val))
    }
}

#[cfg(test)]
mod entity_should {

    use super::Entity;

    fn deserialize_as(expected: Entity, from: &'static str) {
        let entity: Entity = ::serde_json::from_str(from).unwrap();
        assert_eq!(expected, entity);
    }

    #[test]
    fn deserialize_from_0_as_empty() {
        deserialize_as(Entity::Empty, "0");
    }

    #[test]
    fn deserialize_from_1_as_banana() {
        deserialize_as(Entity::Banana, "1");
    }

    #[test]
    fn deserialize_from_2_as_wall() {
        deserialize_as(Entity::Wall, "2");
    }

    #[test]
    fn deserialize_from_3_as_player_3() {
        deserialize_as(Entity::Player(3), "3");
    }
}