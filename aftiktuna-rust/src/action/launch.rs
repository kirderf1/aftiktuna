use crate::action::item;
use crate::action::item::FuelCan;
use crate::area::{Ship, ShipStatus};
use crate::position::Pos;
use crate::view::DisplayInfo;
use hecs::{Entity, World};

pub fn perform(world: &mut World, performer: Entity) -> Option<String> {
    let area = world.get::<&Pos>(performer).ok()?.get_area();
    let name = DisplayInfo::find_definite_name(world, performer);

    let status = world.get::<&Ship>(area).ok()?.status;

    let (new_status, message) = match status {
        ShipStatus::NeedTwoCans => on_need_two_cans(world, performer, &name),
        ShipStatus::NeedOneCan => on_need_one_can(world, performer, &name),
        ShipStatus::Launching => (
            ShipStatus::Launching,
            "The ship is already launching".to_string(),
        ),
    };

    if status != new_status {
        world.get::<&mut Ship>(area).unwrap().status = new_status; //The ship area should still exist since it existed before
    }

    Some(message)
}

fn on_need_two_cans(world: &mut World, aftik: Entity, name: &str) -> (ShipStatus, String) {
    item::consume_one::<FuelCan>(world, aftik).map_or_else(
        || {
            (
                ShipStatus::NeedTwoCans,
                format!("Two fuel cans are needed to launch the ship."),
            )
        },
        |_| on_need_one_can(world, aftik, name),
    )
}

fn on_need_one_can(world: &mut World, aftik: Entity, name: &str) -> (ShipStatus, String) {
    item::consume_one::<FuelCan>(world, aftik).map_or_else(
        || {
            (
                ShipStatus::NeedOneCan,
                format!("{} still need one more fuel can to launch the ship.", name),
            )
        },
        |_| {
            (
                ShipStatus::Launching,
                format!("{} refueled the ship, and set it to launch.", name),
            )
        },
    )
}
