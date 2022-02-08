use specs::{prelude::*, storage::BTreeStorage, Component};

use crate::view::GOType;
use crate::FuelCan;

pub type Coord = usize;

#[derive(Component, Debug)]
#[storage(BTreeStorage)]
pub struct Area {
    pub size: Coord,
    pub label: String,
}

pub fn init_area(world: &mut World) -> Entity {
    let room = world
        .create_entity()
        .with(Area {
            size: 5,
            label: "Room".to_string(),
        })
        .build();

    let pos = Position::new(room, 1, &world.read_storage());
    let aftik = world
        .create_entity()
        .with(GOType::new('A', "Aftik"))
        .with(pos)
        .build();
    place_fuel(world, room, 4);
    place_fuel(world, room, 4);
    aftik
}

fn place_fuel(world: &mut World, area: Entity, coord: Coord) {
    let pos = Position::new(area, coord, &world.read_storage());
    world
        .create_entity()
        .with(GOType::new('f', "Fuel can"))
        .with(pos)
        .with(FuelCan)
        .build();
}

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
