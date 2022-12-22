use crate::action::door::BlockType;
use crate::area::template::LocationData;

#[allow(dead_code)]
pub fn misc_test() -> LocationData {
    let mut location = LocationData::new();

    location.area(
        "Room",
        &["<k", "v", "^", ">c"],
        &[('<', "left"), ('^', "mid"), ('>', "right")],
    );
    location.area(
        "Side Room",
        &["", "<", ">", "f", "f"],
        &[('<', "left"), ('>', "side")],
    );
    location.area(
        "Side Room",
        &["b", "", "", "G", "", "<", "", "", ">", "", "", ""],
        &[('<', "main_right"), ('>', "side")],
    );
    location.area("Room", &["", "^", "", "", ""], &[('^', "mid")]);

    location.blocked_door("left", BlockType::Stuck);
    location.door("right");
    location.blocked_door("side", BlockType::Sealed);
    location.blocked_door("mid", BlockType::Locked);

    location
}

#[allow(dead_code)]
pub fn combat_test() -> LocationData {
    let mut location = LocationData::new();

    location.area(
        "Armory",
        &["Ks", "<", "v", "^", ">", "cB"],
        &[('<', "goblin"), ('^', "eyesaur"), ('>', "azureclops")],
    );
    location.area("Goblin Room", &["G", "", "^", "G", ""], &[('^', "goblin")]);
    location.area("Eyesaur Room", &["", "", "^", "", "E"], &[('^', "eyesaur")]);
    location.area(
        "Azureclops Room",
        &["", "", "^", "", "Z"],
        &[('^', "azureclops")],
    );

    location.door("goblin");
    location.door("eyesaur");
    location.door("azureclops");

    location
}
