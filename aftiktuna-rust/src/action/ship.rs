use crate::action::{self, Error};
use crate::asset::GameAssets;
use crate::core::area::{self, FuelAmount, ShipControls, ShipState, ShipStatus};
use crate::core::item::ItemTypeId;
use crate::core::name::{NameData, NameQuery};
use crate::core::position::{self, Placement, PlacementQuery, Pos};
use crate::core::{CrewMember, inventory};
use crate::game_loop::GameState;
use crate::view::text;
use hecs::{Entity, World};

pub fn refuel(context: &mut action::Context, performer: Entity) -> action::Result {
    let assets = context.view_context.view_buffer.assets;
    let state = &mut context.state;
    let area = state.world.get::<&Pos>(performer).unwrap().get_area();

    let (status, controls_placement) = lookup_ship_state(state, area)?;

    position::move_adjacent_placement(&mut state.world, performer, controls_placement, assets)?;

    let name = NameData::find(&state.world, performer, assets).definite();
    let (new_status, message) = match status {
        ShipStatus::NeedFuel(amount) => match try_refuel(amount, &mut state.world, performer) {
            RefuelResult::Incomplete(amount) => (
                ShipStatus::NeedFuel(amount),
                format!("{name} refueled the ship."),
            ),
            RefuelResult::Complete => (ShipStatus::Refueled, format!("{name} refueled the ship.")),
        },
        ShipStatus::Refueled => {
            return Err(Error::visible(format!(
                "{name} goes to refuel the ship, but sees that it is already refueled."
            )));
        }
        ShipStatus::Launching => return Ok(action::Success),
    };

    if status != new_status {
        //The ship area should still exist since it existed before
        state
            .world
            .get::<&mut ShipState>(state.ship_core)
            .unwrap()
            .status = new_status;
    }

    context.view_context.add_message_at(area, message, state);
    Ok(action::Success)
}

pub fn launch(context: &mut action::Context, performer: Entity) -> action::Result {
    let assets = context.view_context.view_buffer.assets;
    let state = &mut context.state;
    if state.generation_state.is_at_fortuna() {
        return Err(Error::private(
            "The crew won't leave until they find the treasure here.",
        ));
    }

    let area = state.world.get::<&Pos>(performer).unwrap().get_area();

    let (status, controls_placement) = lookup_ship_state(state, area)?;

    position::move_adjacent_placement(&mut state.world, performer, controls_placement, assets)?;

    let (new_status, message) = match status {
        ShipStatus::NeedFuel(amount) => refuel_then_launch(state, performer, amount, assets),
        ShipStatus::Refueled => (
            ShipStatus::Launching,
            format!(
                "{} set the ship to launch.",
                NameData::find(&state.world, performer, assets).definite(),
            ),
        ),
        ShipStatus::Launching => (
            ShipStatus::Launching,
            "The ship is already launching.".to_string(),
        ),
    };

    if status != new_status {
        //The ship area should still exist since it existed before
        state
            .world
            .get::<&mut ShipState>(state.ship_core)
            .unwrap()
            .status = new_status;
    }

    context.view_context.add_message_at(area, message, state);
    Ok(action::Success)
}

fn refuel_then_launch(
    state: &mut GameState,
    performer: Entity,
    amount: FuelAmount,
    assets: &GameAssets,
) -> (ShipStatus, String) {
    let name = NameData::find(&state.world, performer, assets).definite();
    match try_refuel(amount, &mut state.world, performer) {
        RefuelResult::Incomplete(new_amount) => (
            ShipStatus::NeedFuel(new_amount),
            if new_amount != amount {
                format!("{name} refueled the ship.")
            } else {
                incomplete_refuel_message(new_amount, &name)
            },
        ),
        RefuelResult::Complete => {
            let absent_crew = state
                .world
                .query::<(&Pos, NameQuery)>()
                .with::<&CrewMember>()
                .iter()
                .filter(|&(_, (pos, _))| !area::is_in_ship(*pos, &state.world))
                .map(|(_, (_, query))| NameData::from_query(query, assets).definite())
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

fn lookup_ship_state(state: &GameState, area: Entity) -> Result<(ShipStatus, Placement), String> {
    let status = state
        .world
        .get::<&ShipState>(state.ship_core)
        .map_err(|_| "The crew has no ship.".to_string())?
        .status;

    let controls_pos = state
        .world
        .query::<PlacementQuery>()
        .with::<&ShipControls>()
        .iter()
        .map(|(_, query)| Placement::from(query))
        .find(|placement| placement.pos.is_in(area) && area::is_ship(area, &state.world))
        .ok_or_else(|| "Must be in the ship control room to do this.".to_string())?;

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
        && inventory::consume_one(ItemTypeId::is_fuel_can, world, performer).is_none()
    {
        return RefuelResult::Incomplete(FuelAmount::TwoCans);
    }

    if inventory::consume_one(ItemTypeId::is_fuel_can, world, performer).is_none() {
        return RefuelResult::Incomplete(FuelAmount::OneCan);
    }

    RefuelResult::Complete
}
