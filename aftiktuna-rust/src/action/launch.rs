use crate::action;
use crate::action::CrewMember;
use crate::area::{Ship, ShipStatus};
use crate::core::item::FuelCan;
use crate::core::position::Pos;
use crate::core::{inventory, GameState};
use crate::view::name::{NameData, NameQuery};
use hecs::Entity;

pub fn perform(state: &mut GameState, performer: Entity) -> action::Result {
    if state.locations.is_at_fortuna() {
        return Err("You can't leave fortuna yet!".to_string());
    }

    let area = state.world.get::<&Pos>(performer).unwrap().get_area();
    let name = NameData::find(&state.world, performer).definite();

    let status = state
        .world
        .get::<&Ship>(area)
        .map_err(|_| "Tried to launch the ship without being in the ship.".to_string())?
        .status;

    let (new_status, message) = match status {
        ShipStatus::NeedTwoCans => on_need_two_cans(state, performer, &name),
        ShipStatus::NeedOneCan => on_need_one_can(state, performer, &name),
        ShipStatus::Refueled => (
            ShipStatus::Launching,
            format!("{name} set the ship to launch.",),
        ),
        ShipStatus::Launching => (
            ShipStatus::Launching,
            "The ship is already launching.".to_string(),
        ),
    };

    if status != new_status {
        state.world.get::<&mut Ship>(area).unwrap().status = new_status; //The ship area should still exist since it existed before
    }

    action::ok(message)
}

fn on_need_two_cans(state: &mut GameState, performer: Entity, name: &str) -> (ShipStatus, String) {
    inventory::consume_one::<&FuelCan>(&mut state.world, performer).map_or_else(
        || {
            (
                ShipStatus::NeedTwoCans,
                format!("{name} need two fuel cans to launch the ship."),
            )
        },
        |_| on_need_one_can(state, performer, name),
    )
}

fn on_need_one_can(state: &mut GameState, performer: Entity, name: &str) -> (ShipStatus, String) {
    inventory::consume_one::<&FuelCan>(&mut state.world, performer).map_or_else(
        || {
            (
                ShipStatus::NeedOneCan,
                format!("{name} still need one more fuel can to launch the ship."),
            )
        },
        |_| {
            let absent_crew = state.world.query::<(&Pos, NameQuery)>().with::<&CrewMember>().iter().filter(|(_, (pos, _))| !pos.is_in(state.ship)).map(|(_, (_, query))| NameData::from(query).definite()).collect::<Vec<_>>();
            if absent_crew.is_empty() {
                (
                    ShipStatus::Launching,
                    format!("{name} refueled the ship, and set it to launch."),
                )
            } else {
                (
                    ShipStatus::Refueled,
                    format!("{name} refueled the ship. Warning: not all crew members have boarded the ship yet. The following are absent: {}", absent_crew.join(", ")),
                )
            }
        },
    )
}
