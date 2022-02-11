use specs::{prelude::*, storage::BTreeStorage, Component};

use crate::view::DisplayInfo;
use crate::{Door, FuelCan};

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
            size: 3,
            label: "Room".to_string(),
        })
        .build();
    let side_room = world
        .create_entity()
        .with(Area {
            size: 5,
            label: "Side Room".to_string(),
        })
        .build();

    let aftik = place_aftik(world, room, 1);
    place_doors(world, room, 0, side_room, 1);
    place_fuel(world, side_room, 4);
    place_fuel(world, side_room, 4);
    aftik
}

fn place_aftik(world: &mut World, area: Entity, coord: Coord) -> Entity {
    let pos = Pos::new(area, coord, &world.read_storage());
    world
        .create_entity()
        .with(DisplayInfo::new('A', "Aftik", 10))
        .with(Position(pos))
        .build()
}

fn place_doors(world: &mut World, area1: Entity, coord1: Coord, area2: Entity, coord2: Coord) {
    place_door(world, area1, coord1, area2, coord2);
    place_door(world, area2, coord2, area1, coord1);
}

fn place_door(world: &mut World, area: Entity, coord: Coord, dest_area: Entity, dest_coord: Coord) {
    let pos = Pos::new(area, coord, &world.read_storage());
    let dest = Pos::new(dest_area, dest_coord, &world.read_storage());
    world
        .create_entity()
        .with(DisplayInfo::new('^', "Door", 20))
        .with(Position(pos))
        .with(Door { destination: dest })
        .build();
}

fn place_fuel(world: &mut World, area: Entity, coord: Coord) {
    let pos = Pos::new(area, coord, &world.read_storage());
    world
        .create_entity()
        .with(DisplayInfo::new('f', "Fuel can", 1))
        .with(Position(pos))
        .with(FuelCan)
        .build();
}

#[derive(Clone, Debug)]
pub struct Pos {
    area: Entity,
    coord: Coord,
}

impl Pos {
    pub fn new(area: Entity, coord: Coord, storage: &ReadStorage<Area>) -> Pos {
        assert_valid_coord(area, coord, storage);
        Pos { coord, area }
    }

    pub fn get_coord(&self) -> Coord {
        self.coord
    }

    pub fn get_area(&self) -> Entity {
        self.area
    }
}

#[derive(Component, Debug)]
#[storage(BTreeStorage)]
pub struct Position(pub(crate) Pos);

impl Position {
    pub fn move_to(&mut self, new_coord: Coord, storage: &ReadStorage<Area>) {
        self.0 = Pos::new(self.0.get_area(), new_coord, storage);
    }

    pub fn get_coord(&self) -> Coord {
        self.0.get_coord()
    }

    pub fn get_area(&self) -> Entity {
        self.0.get_area()
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
