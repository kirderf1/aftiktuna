use crate::action::door::{BlockType, DoorBlocking};
use crate::area;
use crate::area::Area;
use hecs::{Entity, World};
use crate::position::Coord;

#[allow(dead_code)]
pub fn misc_test(world: &mut World) -> (Entity, Coord) {
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

    area::place_doors(
        world,
        room,
        0,
        area::left_door(),
        side_room,
        1,
        area::left_door(),
        (DoorBlocking(BlockType::Stuck),),
    );
    area::place_doors(
        world,
        room,
        3,
        area::right_door(),
        side_room_2,
        5,
        area::left_door(),
        (),
    );
    area::place_doors(
        world,
        side_room,
        2,
        area::right_door(),
        side_room_2,
        8,
        area::right_door(),
        (DoorBlocking(BlockType::Sealed),),
    );
    area::place_doors(
        world,
        room,
        2,
        area::door(),
        mid_room,
        1,
        area::door(),
        (DoorBlocking(BlockType::Locked),),
    );

    area::place_fuel(world, side_room, 4);
    area::place_fuel(world, side_room, 3);
    area::place_crowbar(world, room, 3);
    area::place_blowtorch(world, side_room_2, 0);
    area::place_keycard(world, room, 0);
    area::place_goblin(world, side_room_2, 3);
    (room, 1)
}

#[allow(dead_code)]
pub fn combat_test(world: &mut World) -> (Entity, Coord) {
    let armory = world.spawn((Area {
        size: 6,
        label: "Armory".to_string(),
    },));
    let goblin_room = world.spawn((Area {
        size: 5,
        label: "Goblin Room".to_string(),
    },));

    area::place_doors(
        world,
        armory,
        1,
        area::door(),
        goblin_room,
        2,
        area::door(),
        (),
    );

    area::place_goblin(world, goblin_room, 0);
    area::place_goblin(world, goblin_room, 3);
    area::place_crowbar(world, armory, 5);
    area::place_bat(world, armory, 5);
    area::place_knife(world, armory, 0);
    area::place_sword(world, armory, 0);
    (armory, 2)
}
