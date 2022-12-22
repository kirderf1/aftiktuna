use crate::action::door::{BlockType, DoorBlocking};
use crate::area;
use crate::area::template::{AreaData, Builder};
use crate::area::DoorInfo;
use crate::position::Pos;
use hecs::World;

#[allow(dead_code)]
pub fn misc_test(world: &mut World) -> Pos {
    let mut builder = Builder::new(world);
    let room = AreaData::create("Room", &["k", "v", "", "c"]).build(&mut builder);
    let side_room = AreaData::create("Side Room", &["", "", "", "f", "f"]).build(&mut builder);
    let side_room_2 = AreaData::create(
        "Side Room",
        &["b", "", "", "G", "", "", "", "", "", "", "", ""],
    )
    .build(&mut builder);
    let mid_room = AreaData::create("Room", &["", "", "", "", ""]).build(&mut builder);
    let entry = builder.get_entry();

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

    entry
}

#[allow(dead_code)]
pub fn combat_test(world: &mut World) -> Pos {
    let mut builder = Builder::new(world);
    let armory = AreaData::create("Armory", &["Ks", "", "v", "", "", "cB"]).build(&mut builder);
    let goblin_room = AreaData::create("Goblin Room", &["G", "", "", "G", ""]).build(&mut builder);
    let eyesaur_room = AreaData::create("Eyesaur Room", &["", "", "", "", "E"]).build(&mut builder);
    let azureclops_room =
        AreaData::create("Azureclops Room", &["", "", "", "", "Z"]).build(&mut builder);
    let entry = builder.get_entry();

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

    entry
}
