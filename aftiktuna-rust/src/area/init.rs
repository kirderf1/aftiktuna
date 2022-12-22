use crate::action::door::{BlockType, DoorBlocking};
use crate::area;
use crate::area::template::Room;
use crate::area::DoorInfo;
use crate::position::Coord;
use hecs::{Entity, World};

#[allow(dead_code)]
pub fn misc_test(world: &mut World) -> (Entity, Coord) {
    let room = Room::create("Room", &["k", "", "", "c"]).build(world);
    let side_room = Room::create("Side Room", &["", "", "", "f", "f"]).build(world);
    let side_room_2 = Room::create(
        "Side Room",
        &["b", "", "", "G", "", "", "", "", "", "", "", ""],
    )
    .build(world);
    let mid_room = Room::create("Room", &["", "", "", "", ""]).build(world);

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

    (room, 1)
}

#[allow(dead_code)]
pub fn combat_test(world: &mut World) -> (Entity, Coord) {
    let armory = Room::create("Armory", &["Ks", "", "", "", "", "cB"]).build(world);
    let goblin_room = Room::create("Goblin Room", &["G", "", "", "G", ""]).build(world);
    let eyesaur_room = Room::create("Eyesaur Room", &["", "", "", "", "E"]).build(world);
    let azureclops_room = Room::create("Azureclops Room", &["", "", "", "", "Z"]).build(world);

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

    (armory, 2)
}
