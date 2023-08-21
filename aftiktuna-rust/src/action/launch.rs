use crate::action;
use crate::area::{Ship, ShipStatus};
use crate::core::item::FuelCan;
use crate::core::position::Pos;
use crate::core::{inventory, GameState};
use crate::view::NameData;
use hecs::{Entity, World};

pub fn perform(state: &mut GameState, performer: Entity) -> action::Result {
    if state.locations.is_at_fortuna() {
        return Err("You can't leave fortuna yet!".to_string());
    }

    let world = &mut state.world;
    let area = world.get::<&Pos>(performer).unwrap().get_area();
    let name = NameData::find(world, performer).definite();

    let status = world
        .get::<&Ship>(area)
        .map_err(|_| "Tried to launch the ship without being in the ship.".to_string())?
        .status;

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

    action::ok(message)
}

fn on_need_two_cans(world: &mut World, aftik: Entity, name: &str) -> (ShipStatus, String) {
    inventory::consume_one::<&FuelCan>(world, aftik).map_or_else(
        || {
            (
                ShipStatus::NeedTwoCans,
                format!("{} need two fuel cans to launch the ship.", name),
            )
        },
        |_| on_need_one_can(world, aftik, name),
    )
}

fn on_need_one_can(world: &mut World, aftik: Entity, name: &str) -> (ShipStatus, String) {
    inventory::consume_one::<&FuelCan>(world, aftik).map_or_else(
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
