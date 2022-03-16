use crate::action::door::{BlockType, Blowtorch, Crowbar, Door, DoorBlocking, Keycard};
use crate::action::item::{FuelCan, Item};
use crate::view::DisplayInfo;
use hecs::{DynamicBundle, Entity, World};
use crate::action::MovementBlocking;

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
    let mid_room = world.spawn((Area {
        size: 5,
        label: "Room".to_string(),
    },));

    place_doors(
        world,
        room,
        0,
        left_door(),
        side_room,
        1,
        left_door(),
        (DoorBlocking(BlockType::Stuck),),
    );
    place_doors(
        world,
        room,
        3,
        right_door(),
        side_room_2,
        5,
        left_door(),
        (),
    );
    place_doors(
        world,
        side_room,
        2,
        right_door(),
        side_room_2,
        8,
        right_door(),
        (DoorBlocking(BlockType::Sealed),),
    );
    place_doors(
        world,
        room,
        2,
        door(),
        mid_room,
        1,
        door(),
        (DoorBlocking(BlockType::Locked),),
    );

    place_fuel(world, side_room, 4);
    place_fuel(world, side_room, 4);
    place_crowbar(world, room, 3);
    place_blowtorch(world, side_room_2, 0);
    place_keycard(world, room, 0);
    place_goblin(world, side_room_2, 4);
    place_aftik(world, room, 1)
}

fn place_aftik(world: &mut World, area: Entity, coord: Coord) -> Entity {
    let pos = Pos::new(area, coord, world);
    world.spawn((DisplayInfo::new('A', "Aftik", 10), Position(pos)))
}

fn place_goblin(world: &mut World, area: Entity, coord: Coord) -> Entity {
    let pos = Pos::new(area, coord, world);
    world.spawn((DisplayInfo::new('G', "Goblin", 10), Position(pos), MovementBlocking))
}

fn place_doors(
    world: &mut World,
    area1: Entity,
    coord1: Coord,
    disp1: DisplayInfo,
    area2: Entity,
    coord2: Coord,
    disp2: DisplayInfo,
    pair_components: impl DynamicBundle,
) {
    let pos1 = Pos::new(area1, coord1, world);
    let pos2 = Pos::new(area2, coord2, world);
    let door_pair = world.spawn(pair_components);
    place_door(world, pos1, disp1, pos2, door_pair);
    place_door(world, pos2, disp2, pos1, door_pair);
}

fn place_door(
    world: &mut World,
    pos: Pos,
    disp: DisplayInfo,
    destination: Pos,
    door_pair: Entity,
) -> Entity {
    world.spawn((
        disp,
        Position(pos),
        Door {
            destination,
            door_pair,
        },
    ))
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

fn place_crowbar(world: &mut World, area: Entity, coord: Coord) {
    let pos = Pos::new(area, coord, world);
    world.spawn((
        DisplayInfo::new('c', "Crowbar", 1),
        Position(pos),
        Item,
        Crowbar,
    ));
}

fn place_blowtorch(world: &mut World, area: Entity, coord: Coord) {
    let pos = Pos::new(area, coord, world);
    world.spawn((
        DisplayInfo::new('b', "Blowtorch", 1),
        Position(pos),
        Item,
        Blowtorch,
    ));
}

fn place_keycard(world: &mut World, area: Entity, coord: Coord) {
    let pos = Pos::new(area, coord, world);
    world.spawn((
        DisplayInfo::new('k', "Keycard", 1),
        Position(pos),
        Item,
        Keycard,
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
    pub fn get_coord(&self) -> Coord {
        self.0.get_coord()
    }

    pub fn get_area(&self) -> Entity {
        self.0.get_area()
    }

    pub fn is_in(&self, area: Entity) -> bool {
        self.get_area().eq(&area)
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
