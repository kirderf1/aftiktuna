use crate::action::door::BlockType;
use crate::area::door::DoorType;
use crate::area::template::LocationData;

#[allow(dead_code)]
pub fn misc_test() -> LocationData {
    let mut location = LocationData::new();

    location
        .area("Room", &["<k", "v", "^", ">c"])
        .door_symbol('<', DoorType::LeftDoor, "left")
        .door_symbol('^', DoorType::MidDoor, "mid")
        .door_symbol('>', DoorType::RightDoor, "right");
    location
        .area("Side Room", &["", "<", ">", "f", "f"])
        .door_symbol('<', DoorType::LeftDoor, "left")
        .door_symbol('>', DoorType::RightDoor, "side");
    location
        .area(
            "Side Room",
            &["b", "", "", "G", "", "<", "", "", ">", "", "", ""],
        )
        .door_symbol('<', DoorType::LeftDoor, "right")
        .door_symbol('>', DoorType::RightDoor, "side");
    location
        .area("Room", &["", "^", "", "", ""])
        .door_symbol('^', DoorType::Door, "mid");

    location.blocked_door("left", BlockType::Stuck);
    location.door("right");
    location.blocked_door("side", BlockType::Sealed);
    location.blocked_door("mid", BlockType::Locked);

    location
}

#[allow(dead_code)]
pub fn combat_test() -> LocationData {
    let mut location = LocationData::new();

    location
        .area("Armory", &["Ks", "<", "v", "^", ">", "cB"])
        .door_symbol('<', DoorType::LeftDoor, "goblin")
        .door_symbol('^', DoorType::MidDoor, "eyesaur")
        .door_symbol('>', DoorType::RightDoor, "azureclops");
    location
        .area("Goblin Room", &["G", "", "^", "G", ""])
        .door_symbol('^', DoorType::Door, "goblin");
    location
        .area("Eyesaur Room", &["", "", "^", "", "E"])
        .door_symbol('^', DoorType::Door, "eyesaur");
    location
        .area("Azureclops Room", &["", "", "^", "", "Z"])
        .door_symbol('^', DoorType::Door, "azureclops");

    location.door("goblin");
    location.door("eyesaur");
    location.door("azureclops");

    location
}

#[allow(dead_code)]
pub fn abandoned_facility() -> LocationData {
    let mut location = LocationData::new();

    location
        .area("Field in front of a building", &["v", "", "^", "", "", ">"])
        .door_symbol('^', DoorType::Door, "entrance")
        .door_symbol('>', DoorType::RightPath, "path");
    location
        .area("Field", &["^", "", "", "k", ""])
        .door_symbol('^', DoorType::Path, "path");
    location
        .area("Entrance hall", &["", "<", "", "^", "", ">", ""])
        .door_symbol('<', DoorType::LeftDoor, "sealed")
        .door_symbol('^', DoorType::MidDoor, "corridor1")
        .door_symbol('>', DoorType::RightDoor, "entrance");
    location
        .area("Corridor", &["<", "", "^", "E", ">"])
        .door_symbol('<', DoorType::LeftDoor, "corridor1")
        .door_symbol('^', DoorType::MidDoor, "room1")
        .door_symbol('>', DoorType::RightDoor, "corridor2");
    location
        .area("Corridor", &["<", "", "^", "", ">"])
        .door_symbol('<', DoorType::LeftDoor, "corridor2")
        .door_symbol('^', DoorType::MidDoor, "room2")
        .door_symbol('>', DoorType::RightDoor, "room3");
    location
        .area("Room", &["", "c", "", "^"])
        .door_symbol('^', DoorType::Door, "room1");
    location
        .area("Room", &["b", "", "", "^"])
        .door_symbol('^', DoorType::Door, "room2");
    location
        .area("Room", &["^", "E", "", "f"])
        .door_symbol('^', DoorType::Door, "room3");
    location
        .area("Room", &["ff", "Z", "^", "s"])
        .door_symbol('^', DoorType::Door, "sealed");

    location.door("path");
    location.blocked_door("entrance", BlockType::Locked);
    location.blocked_door("sealed", BlockType::Sealed);
    location.door("corridor1");
    location.door("corridor2");
    location.door("room1");
    location.blocked_door("room2", BlockType::Stuck);
    location.door("room3");

    location
}
