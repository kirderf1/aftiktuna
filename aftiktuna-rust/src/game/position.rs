use crate::Area;
use specs::{storage::BTreeStorage, Component, Entity, ReadStorage};

pub type Coord = usize;

#[derive(Component, Debug, Clone)]
#[storage(BTreeStorage)]
pub struct Position {
    area: Entity,
    coord: Coord,
}

impl Position {
    pub fn new(area: Entity, coord: Coord, storage: &ReadStorage<Area>) -> Position {
        assert_valid_coord(area, coord, storage);
        Position { coord, area }
    }

    pub fn get_coord(&self) -> Coord {
        self.coord
    }

    pub fn get_area(&self) -> Entity {
        self.area
    }

    pub fn move_to(&mut self, new_coord: Coord, storage: &ReadStorage<Area>) {
        assert_valid_coord(self.area, new_coord, storage);
        self.coord = new_coord;
    }
}

fn assert_valid_coord(area: Entity, coord: Coord, storage: &ReadStorage<Area>) {
    let area_size = storage
        .get(area)
        .expect("Expected given area to have an area component")
        .size;
    if coord >= area_size {
        panic!(
            "Position {} is out of bounds for room with size {}.",
            coord, area_size
        );
    }
}
