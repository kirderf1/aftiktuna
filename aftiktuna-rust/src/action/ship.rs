use crate::action::{self, Error};
use crate::core::area::{self, FuelAmount, ShipControls, ShipState, ShipStatus};
use crate::core::item::FuelCan;
use crate::core::name::{NameData, NameQuery};
use crate::core::position::{self, Pos};
use crate::core::{CrewMember, inventory};
use crate::game_loop::GameState;
use crate::view::text;
use hecs::{Entity, World};

pub fn refuel(state: &mut GameState, performer: Entity) -> action::Result {
    let area = state.world.get::<&Pos>(performer).unwrap().get_area();

    let (status, controls_pos) = lookup_ship_state(&state.world, area)?;

    position::move_adjacent(&mut state.world, performer, controls_pos)?;

    let name = NameData::find(&state.world, performer).definite();
    let (new_status, message) = match status {
        ShipStatus::NeedFuel(amount) => match try_refuel(amount, &mut state.world, performer) {
            RefuelResult::Incomplete(amount) => (
                ShipStatus::NeedFuel(amount),
                incomplete_refuel_message(amount, &name),
            ),
            RefuelResult::Complete => (ShipStatus::Refueled, format!("{name} refueled the ship.")),
        },
        ShipStatus::Refueled => {
            return Err(Error::visible(format!(
                "{name} goes to refuel the ship, but sees that it is already refueled."
            )));
        }
        ShipStatus::Launching => return action::silent_ok(),
    };

    if status != new_status {
        //The ship area should still exist since it existed before
        state.world.get::<&mut ShipState>(area).unwrap().status = new_status;
    }

    action::ok(message)
}

pub fn launch(state: &mut GameState, performer: Entity) -> action::Result {
    if state.generation_state.is_at_fortuna() {
        return Err(Error::private(
            "The crew won't leave until they find the treasure here.",
        ));
    }

    let area = state.world.get::<&Pos>(performer).unwrap().get_area();

    let (status, controls_pos) = lookup_ship_state(&state.world, area)?;

    position::move_adjacent(&mut state.world, performer, controls_pos)?;

    let (new_status, message) = match status {
        ShipStatus::NeedFuel(amount) => refuel_then_launch(state, performer, amount),
        ShipStatus::Refueled => (
            ShipStatus::Launching,
            format!(
                "{} set the ship to launch.",
                NameData::find(&state.world, performer).definite()
            ),
        ),
        ShipStatus::Launching => (
            ShipStatus::Launching,
            "The ship is already launching.".to_string(),
        ),
    };

    if status != new_status {
        //The ship area should still exist since it existed before
        state.world.get::<&mut ShipState>(area).unwrap().status = new_status;
    }

    action::ok(message)
}

fn refuel_then_launch(
    state: &mut GameState,
    performer: Entity,
    amount: FuelAmount,
) -> (ShipStatus, String) {
    let name = NameData::find(&state.world, performer).definite();
    match try_refuel(amount, &mut state.world, performer) {
        RefuelResult::Incomplete(amount) => (
            ShipStatus::NeedFuel(amount),
            incomplete_refuel_message(amount, &name),
        ),
        RefuelResult::Complete => {
            let absent_crew = state
                .world
                .query::<(&Pos, NameQuery)>()
                .with::<&CrewMember>()
                .iter()
                .filter(|&(_, (pos, _))| !area::is_in_ship(*pos, &state.world))
                .map(|(_, (_, query))| NameData::from(query).definite())
                .collect::<Vec<_>>();
            if absent_crew.is_empty() {
                (
                    ShipStatus::Launching,
                    format!("{name} refueled the ship, and set it to launch."),
                )
            } else {
                (
                    ShipStatus::Refueled,
                    format!(
                        "{name} refueled the ship. Warning: not all crew members have boarded the ship yet. {absent_crew} are still absent.",
                        absent_crew = text::join_elements(absent_crew)
                    ),
                )
            }
        }
    }
}

fn lookup_ship_state(world: &World, area: Entity) -> Result<(ShipStatus, Pos), String> {
    let status = world
        .get::<&ShipState>(area)
        .map_err(|_| "Must be in the ship control room to do this.".to_string())?
        .status;

    let controls_pos = world
        .query::<&Pos>()
        .with::<&ShipControls>()
        .iter()
        .map(|(_, pos)| *pos)
        .find(|pos| pos.is_in(area))
        .ok_or_else(|| "The ship is missing its controls.".to_string())?;

    Ok((status, controls_pos))
}

enum RefuelResult {
    Complete,
    Incomplete(FuelAmount),
}

fn incomplete_refuel_message(amount: FuelAmount, name: &str) -> String {
    match amount {
        FuelAmount::TwoCans => format!("{name} need two fuel cans to refuel the ship."),
        FuelAmount::OneCan => format!("{name} still need one more fuel can to refuel the ship."),
    }
}

fn try_refuel(amount: FuelAmount, world: &mut World, performer: Entity) -> RefuelResult {
    if amount == FuelAmount::TwoCans
        && inventory::consume_one::<&FuelCan>(world, performer).is_none()
    {
        return RefuelResult::Incomplete(FuelAmount::TwoCans);
    }

    if inventory::consume_one::<&FuelCan>(world, performer).is_none() {
        return RefuelResult::Incomplete(FuelAmount::OneCan);
    }

    RefuelResult::Complete
}
