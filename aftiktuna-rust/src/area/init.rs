use crate::action::door::{BlockType, DoorBlocking};
use crate::area;
use crate::area::{Area, DoorInfo, item};
use crate::position::Coord;
use hecs::{Entity, World};

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
        DoorInfo(room, 0, area::left_door()),
        DoorInfo(side_room, 1, area::left_door()),
        (DoorBlocking(BlockType::Stuck),),
    );
    area::place_doors(
        world,
        DoorInfo(room, 3, area::right_door()),
        DoorInfo(side_room_2, 5, area::left_door()),
        (),
    );
    area::place_doors(
        world,
        DoorInfo(side_room, 2, area::right_door()),
        DoorInfo(side_room_2, 8, area::right_door()),
        (DoorBlocking(BlockType::Sealed),),
    );
    area::place_doors(
        world,
        DoorInfo(room, 2, area::door()),
        DoorInfo(mid_room, 1, area::door()),
        (DoorBlocking(BlockType::Locked),),
    );

    item::place_fuel(world, side_room, 4);
    item::place_fuel(world, side_room, 3);
    item::place_crowbar(world, room, 3);
    item::place_blowtorch(world, side_room_2, 0);
    item::place_keycard(world, room, 0);
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
    let eyesaur_room = world.spawn((Area {
        size: 5,
        label: "Eyesaur Room".to_string(),
    },));
    let azureclops_room = world.spawn((Area {
        size: 5,
        label: "Azureclops Room".to_string(),
    },));

    area::place_doors(
        world,
        DoorInfo(armory, 1, area::left_door()),
        DoorInfo(goblin_room, 2, area::door()),
        (),
    );
    area::place_doors(
        world,
        DoorInfo(armory, 3, area::door()),
        DoorInfo(eyesaur_room, 2, area::door()),
        (),
    );
    area::place_doors(
        world,
        DoorInfo(armory, 4, area::right_door()),
        DoorInfo(azureclops_room, 2, area::door()),
        (),
    );

    area::place_goblin(world, goblin_room, 0);
    area::place_goblin(world, goblin_room, 3);
    area::place_eyesaur(world, eyesaur_room, 4);
    area::place_azureclops(world, azureclops_room, 4);
    item::place_crowbar(world, armory, 5);
    item::place_bat(world, armory, 5);
    item::place_knife(world, armory, 0);
    item::place_sword(world, armory, 0);
    (armory, 2)
}
