use crate::action::{self, Context, Error};
use crate::ai;
use crate::core::behavior::{self, Character, Intention, RepeatingAction};
use crate::core::display::DialogueExpression;
use crate::core::item::Tool;
use crate::core::name::{NameData, NameIdData};
use crate::core::position::{self, Direction, Placement, PlacementQuery, Pos};
use crate::core::status::Stamina;
use crate::core::{BlockType, CrewMember, Door, DoorKind, IsCut, area, inventory};
use crate::game_loop::GameState;
use crate::view::text::CombinableMsgType;
use hecs::{Entity, World};
use std::ops::Deref;

fn check_tool_for_forcing(
    block_type: BlockType,
    world: &World,
    performer: Entity,
    performer_name: &str,
) -> Result<Tool, String> {
    for tool in block_type.usable_tools() {
        if inventory::is_holding_tool(world, performer, tool) {
            return Ok(tool);
        }
    }
    match block_type {
        BlockType::Stuck => Err(format!(
            "{performer_name} needs some sort of tool to force the door open.",
        )),
        BlockType::Sealed => Err(format!(
            "{performer_name} needs some sort of tool to break the door open.",
        )),
    }
}

pub(super) fn enter_door(context: &mut Context, performer: Entity, door: Entity) -> action::Result {
    let action::Context {
        state,
        view_context,
    } = context;
    let assets = view_context.view_buffer.assets;
    let world = &mut state.world;
    let performer_name = NameIdData::find(world, performer);

    let door_pos = *world.get::<&Pos>(door).ok().ok_or_else(|| {
        format!(
            "{the_performer} lost track of the door.",
            the_performer = performer_name.clone().lookup(assets).definite()
        )
    })?;
    if Ok(door_pos.get_area()) != world.get::<&Pos>(performer).map(|pos| pos.get_area()) {
        return Err(Error::private(format!(
            "{the_performer} cannot reach the door from here.",
            the_performer = performer_name.lookup(assets).definite()
        )));
    }

    position::push_and_move(world, performer, door_pos, assets)?;

    let door_data = world
        .get::<&Door>(door)
        .map_err(|_| "The door ceased being a door.".to_string())
        .map(|door| door.deref().clone())?;

    if let Ok(block_type) = world
        .get::<&BlockType>(door_data.door_pair)
        .map(|block_type| *block_type)
    {
        context
            .view_context
            .make_noise_at(&[door_pos.get_area()], state);

        on_door_failure(state, performer, door, block_type);
        return Err(Error::visible(format!(
            "{performer} is unable to enter the door as it is {blocked}.",
            performer = performer_name.lookup(assets).definite(),
            blocked = block_type.description(),
        )));
    }

    view_context.capture_unseen_view(door_pos.get_area(), state);

    let world = &mut state.world;
    let performer_placement =
        Placement::from(world.query_one_mut::<PlacementQuery>(performer).unwrap());
    let destination_pos = if performer_placement.is_large {
        door_data
            .destination
            .try_offset_direction(performer_placement.direction, world)
            .unwrap_or(door_data.destination)
    } else {
        door_data.destination
    };
    if let Err(blockage) = position::check_is_pos_blocked(Some(performer), destination_pos, world) {
        blockage
            .try_push(
                Direction::towards_center(door_data.destination, world),
                world,
            )
            .map_err(|_| blockage.into_message(world, assets))?;
    }
    world.insert_one(performer, destination_pos).unwrap();
    if let Ok(mut stamina) = world.get::<&mut Stamina>(performer) {
        stamina.on_move();
    }
    if performer == state.controlled {
        view_context.view_buffer.mark_unseen_view();
    }

    let door_name = NameIdData::find(world, door);
    let message = match door_data.kind {
        DoorKind::Door => CombinableMsgType::EnterDoor(door, door_name),
        DoorKind::Path => CombinableMsgType::EnterPath(door, door_name),
    }
    .message(performer_name.clone());
    view_context.add_message_at(door_pos.get_area(), message, state);

    view_context.add_message_at(
        door_data.destination.get_area(),
        CombinableMsgType::Arrive(door).message(performer_name),
        state,
    );

    view_context.make_noise_at(
        &[door_pos.get_area(), door_data.destination.get_area()],
        state,
    );

    Ok(action::Success)
}

pub(super) fn force_door(
    context: Context,
    performer: Entity,
    door: Entity,
    assisting: bool,
) -> action::Result {
    let Context {
        state,
        mut view_context,
    } = context;
    let assets = view_context.view_buffer.assets;
    let world = &state.world;
    let performer_name = NameData::find(world, performer, assets).definite();
    let door_pos = *world
        .get::<&Pos>(door)
        .ok()
        .ok_or_else(|| format!("{performer_name} lost track of the door."))?;
    if Ok(door_pos.get_area()) != world.get::<&Pos>(performer).map(|pos| pos.get_area()) {
        return Err(Error::private(format!(
            "{performer_name} cannot reach the door from here."
        )));
    }

    let door_pair = world
        .get::<&Door>(door)
        .map_err(|_| "The door ceased being a door.")?
        .door_pair;

    let movement = position::prepare_move(world, performer, door_pos)
        .map_err(|blockage| blockage.into_message(world, assets))?;
    view_context.capture_frame_for_dialogue(state);
    let world = &mut state.world;
    movement.perform(world).unwrap();
    if assisting {
        view_context.view_buffer.push_dialogue(
            world,
            performer,
            DialogueExpression::Neutral,
            "I'll help you get that door open.",
        );
    }

    let block_type = *world.get::<&BlockType>(door_pair).map_err(|_| {
        Error::visible(format!(
            "{performer_name} inspects the door, but it does not appear to be stuck."
        ))
    })?;

    match check_tool_for_forcing(block_type, world, performer, &performer_name) {
        Err(message) => {
            on_door_failure(state, performer, door, block_type);
            Err(Error::visible(message))
        }
        Ok(tool) => {
            world.remove_one::<BlockType>(door_pair).unwrap();
            if tool == Tool::Blowtorch {
                let doors = world
                    .query::<&Door>()
                    .iter()
                    .filter(|&(_, door)| door.door_pair == door_pair)
                    .map(|(entity, _)| entity)
                    .collect::<Vec<_>>();
                for door in doors {
                    world.insert_one(door, IsCut).unwrap();
                }
            }

            view_context.add_message_at(
                door_pos.get_area(),
                tool.into_message(&performer_name),
                state,
            );
            Ok(action::Success)
        }
    }
}

fn on_door_failure(state: &mut GameState, performer: Entity, door: Entity, block_type: BlockType) {
    let world = &mut state.world;
    let area = world.get::<&Pos>(performer).unwrap().get_area();
    if !behavior::is_safe(world, area) {
        return;
    }

    let crew_member = world
        .query::<&Pos>()
        .with::<(&CrewMember, &Character)>()
        .iter()
        .find(|&(crew_member, pos)| {
            crew_member != state.controlled
                && pos.is_in(area)
                && block_type
                    .usable_tools()
                    .into_iter()
                    .any(|tool| inventory::is_holding_tool(world, crew_member, tool))
        })
        .map(|(crew_member, _)| crew_member);
    if let Some(crew_member) = crew_member {
        world
            .insert_one(crew_member, Intention::Force(door))
            .unwrap();
    }
}

pub(super) fn go_to_ship(mut context: Context, performer: Entity) -> action::Result {
    let world = context.mut_world();
    let area = world.get::<&Pos>(performer).unwrap().get_area();
    if area::is_ship(area, world) {
        return Ok(action::Success);
    }

    let path = ai::find_path_towards(world, area, |area| area::is_ship(area, world))
        .ok_or_else(|| "Could not find a path to the ship.".to_string())?;

    let result = enter_door(&mut context, performer, path);

    let world = context.mut_world();
    let area = world.get::<&Pos>(performer).unwrap().get_area();
    if result.is_ok() && behavior::is_safe(world, area) && !area::is_ship(area, world) {
        world
            .insert_one(performer, RepeatingAction::GoToShip)
            .unwrap();
    }
    result
}
