use crate::action::door::{BlockType, DoorBlocking};
use crate::area;
use crate::area::template::Room;
use crate::area::{creature, item, DoorInfo};
use crate::position::Coord;
use hecs::{Entity, World};

#[allow(dead_code)]
pub fn misc_test(world: &mut World) -> (Entity, Coord) {
    let room = Room::create("Room", vec!["", "", "", ""]).build(world);
    let side_room = Room::create("Side Room", vec!["", "", "", "", ""]).build(world);
    let side_room_2 = Room::create(
        "Side Room",
        vec!["", "", "", "", "", "", "", "", "", "", "", ""],
    )
    .build(world);
    let mid_room = Room::create("Room", vec!["", "", "", "", ""]).build(world);

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
    creature::place_goblin(world, side_room_2, 3);
    (room, 1)
}

#[allow(dead_code)]
pub fn combat_test(world: &mut World) -> (Entity, Coord) {
    let armory = Room::create("Armory", vec!["", "", "", "", "", ""]).build(world);
    let goblin_room = Room::create("Goblin Room", vec!["", "", "", "", ""]).build(world);
    let eyesaur_room = Room::create("Eyesaur Room", vec!["", "", "", "", ""]).build(world);
    let azureclops_room = Room::create("Azureclops Room", vec!["", "", "", "", ""]).build(world);

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

    creature::place_goblin(world, goblin_room, 0);
    creature::place_goblin(world, goblin_room, 3);
    creature::place_eyesaur(world, eyesaur_room, 4);
    creature::place_azureclops(world, azureclops_room, 4);
    item::place_crowbar(world, armory, 5);
    item::place_bat(world, armory, 5);
    item::place_knife(world, armory, 0);
    item::place_sword(world, armory, 0);
    (armory, 2)
}
