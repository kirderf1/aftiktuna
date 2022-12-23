use crate::action::door::BlockType;
use crate::area::template::{DoorType, LocationData};

#[allow(dead_code)]
pub fn misc_test() -> LocationData {
    let mut location = LocationData::new();

    location
        .area("Room", &["<k", "v", "^", ">c"])
        .door_symbol('<', DoorType::LeftDoor, "left")
        .door_symbol('^', DoorType::Door, "mid")
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
        .door_symbol('<', DoorType::LeftDoor, "main_right")
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
        .door_symbol('^', DoorType::Door, "eyesaur")
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
