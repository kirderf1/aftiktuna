use crate::action::{Door, FuelCan, Item};
use crate::view::DisplayInfo;
use hecs::{Entity, World};

pub type Coord = usize;

pub struct Area {
    pub size: Coord,
    pub label: String,
}

pub fn init_area(world: &mut World) -> Entity {
    let room = world.spawn((Area {
        size: 4,
        label: "Room".to_string(),
    },));
    let side_room = world.spawn((Area {
        size: 5,
        label: "Side Room".to_string(),
    },));
    let side_room_2 = world.spawn((Area {
        size: 12,
        label: "Side Room".to_string(),
    },));

    let aftik = place_aftik(world, room, 1);
    place_doors(world, room, 0, left_door(), side_room, 1, door());
    place_doors(world, room, 3, right_door(), side_room_2, 5, door());
    place_fuel(world, side_room, 4);
    place_fuel(world, side_room, 4);
    aftik
}

fn place_aftik(world: &mut World, area: Entity, coord: Coord) -> Entity {
    let pos = Pos::new(area, coord, world);
    world.spawn((DisplayInfo::new('A', "Aftik", 10), Position(pos)))
}

fn place_doors(
    world: &mut World,
    area1: Entity,
    coord1: Coord,
    disp1: DisplayInfo,
    area2: Entity,
    coord2: Coord,
    disp2: DisplayInfo,
) {
    place_door(world, area1, coord1, disp1, area2, coord2);
    place_door(world, area2, coord2, disp2, area1, coord1);
}

fn place_door(
    world: &mut World,
    area: Entity,
    coord: Coord,
    disp: DisplayInfo,
    dest_area: Entity,
    dest_coord: Coord,
) {
    let pos = Pos::new(area, coord, world);
    let dest = Pos::new(dest_area, dest_coord, world);
    world.spawn((disp, Position(pos), Door { destination: dest }));
}

fn door() -> DisplayInfo {
    DisplayInfo::new('^', "Door", 20)
}

fn left_door() -> DisplayInfo {
    DisplayInfo::new('<', "Left door", 20)
}

fn right_door() -> DisplayInfo {
    DisplayInfo::new('>', "Right door", 20)
}

fn place_fuel(world: &mut World, area: Entity, coord: Coord) {
    let pos = Pos::new(area, coord, world);
    world.spawn((
        DisplayInfo::new('f', "Fuel can", 1),
        Position(pos),
        Item,
        FuelCan,
    ));
}

#[derive(Clone, Copy, Debug)]
pub struct Pos {
    area: Entity,
    coord: Coord,
}

impl Pos {
    pub fn new(area: Entity, coord: Coord, world: &World) -> Pos {
        assert_valid_coord(area, coord, world);
        Pos { coord, area }
    }

    pub fn get_coord(&self) -> Coord {
        self.coord
    }

    pub fn get_area(&self) -> Entity {
        self.area
    }
}

#[derive(Debug)]
pub struct Position(pub(crate) Pos);

impl Position {
    pub fn move_to(&mut self, new_coord: Coord, world: &World) {
        self.0 = Pos::new(self.0.get_area(), new_coord, world);
    }

    pub fn get_coord(&self) -> Coord {
        self.0.get_coord()
    }

    pub fn get_area(&self) -> Entity {
        self.0.get_area()
    }
}

fn assert_valid_coord(area: Entity, coord: Coord, world: &World) {
    let area_size = world
        .get::<Area>(area)
        .expect("Expected given area to have an area component")
        .size;
    assert!(
        coord < area_size,
        "Position {} is out of bounds for room with size {}.",
        coord,
        area_size
    );
}
