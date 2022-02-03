use specs::{Component, storage::BTreeStorage};

use crate::game::AREA_SIZE;

pub type Coord = usize;

#[derive(Component, Debug)]
#[storage(BTreeStorage)]
pub struct Position {
    coord: Coord,
}

impl Position {
    pub fn new(coord: Coord) -> Position {
        assert_valid_coord(coord);
        Position { coord }
    }

    pub fn get_coord(&self) -> Coord {
        self.coord
    }

    pub fn move_to(&mut self, new_coord: Coord) {
        assert_valid_coord(new_coord);
        self.coord = new_coord;
    }
}

fn assert_valid_coord(coord: Coord) {
    if coord >= AREA_SIZE {
        panic!(
            "Position {} is out of bounds for room with size {}.",
            coord, AREA_SIZE
        );
    }
}
