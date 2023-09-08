use crate::action::{Context, CrewMember};
use crate::core::ai::Intention;
use crate::core::item::{Keycard, Tool};
use crate::core::position::{Blockage, Pos};
use crate::core::{inventory, position};
use crate::view::name::NameData;
use crate::view::TextureType;
use crate::{action, core};
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Door {
    pub kind: DoorKind,
    pub destination: Pos,
    pub door_pair: Entity,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum DoorKind {
    Door,
    Path,
}

impl DoorKind {
    fn get_move_message(self, performer: &str) -> String {
        match self {
            DoorKind::Door => format!("{} entered the door into a new area.", performer),
            DoorKind::Path => format!("{} followed the path to a new area.", performer),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockType {
    Stuck,
    Sealed,
    Locked,
}

impl BlockType {
    pub fn description(self) -> &'static str {
        match self {
            BlockType::Stuck => "stuck",
            BlockType::Sealed => "sealed shut",
            BlockType::Locked => "locked",
        }
    }

    fn usable_tools(self) -> Vec<Tool> {
        match self {
            BlockType::Stuck => vec![Tool::Crowbar, Tool::Blowtorch],

            BlockType::Sealed | BlockType::Locked => vec![Tool::Blowtorch],
        }
    }

    fn try_force(self, world: &World, aftik: Entity, aftik_name: &str) -> Result<Tool, String> {
        for tool in self.usable_tools() {
            if inventory::is_holding_tool(world, aftik, tool) {
                return Ok(tool);
            }
        }
        match self {
            BlockType::Stuck => Err(format!(
                "{aftik_name} needs some sort of tool to force the door open.",
            )),
            BlockType::Sealed | BlockType::Locked => Err(format!(
                "{aftik_name} needs some sort of tool to break the door open.",
            )),
        }
    }
}

pub fn enter_door(world: &mut World, aftik: Entity, door: Entity) -> action::Result {
    let aftik_name = NameData::find(world, aftik).definite();
    let door_pos = *world
        .get::<&Pos>(door)
        .ok()
        .ok_or_else(|| format!("{} lost track of the door.", aftik_name))?;
    if Ok(door_pos.get_area()) != world.get::<&Pos>(aftik).map(|pos| pos.get_area()) {
        return Err(format!("{} cannot reach the door from here.", aftik_name));
    }

    position::move_to(world, aftik, door_pos)?;

    let door_data = world
        .get::<&Door>(door)
        .map_err(|_| "The door ceased being a door.".to_string())
        .map(|door| door.deref().clone())?;

    let used_keycard = if let Ok(block_type) = world
        .get::<&BlockType>(door_data.door_pair)
        .map(|block_type| *block_type)
    {
        if block_type == BlockType::Locked && inventory::is_holding::<&Keycard>(world, aftik) {
            true
        } else {
            on_door_failure(world, aftik, door, block_type);
            return Err(format!("The door is {}.", block_type.description()));
        }
    } else {
        false
    };

    world.insert_one(aftik, door_data.destination).unwrap();
    let areas = vec![door_pos.get_area(), door_data.destination.get_area()];
    if used_keycard {
        action::ok_at(
            format!(
                "Using their keycard, {}",
                door_data.kind.get_move_message(&aftik_name),
            ),
            areas,
        )
    } else {
        action::ok_at(door_data.kind.get_move_message(&aftik_name), areas)
    }
}

pub(super) fn force_door(
    mut context: Context,
    performer: Entity,
    door: Entity,
    assisting: bool,
) -> action::Result {
    let world = context.mut_world();
    let performer_name = NameData::find(world, performer).definite();
    let door_pos = *world
        .get::<&Pos>(door)
        .ok()
        .ok_or_else(|| format!("{performer_name} lost track of the door."))?;
    if Ok(door_pos.get_area()) != world.get::<&Pos>(performer).map(|pos| pos.get_area()) {
        return Err(format!("{performer_name} cannot reach the door from here."));
    }

    let movement =
        position::prepare_move(world, performer, door_pos).map_err(Blockage::into_message)?;
    context.capture_frame_for_dialogue();
    movement.perform(context.mut_world()).unwrap();
    if assisting {
        context.add_dialogue(performer, "I'll help you get that door open.");
    }
    let world = context.mut_world();

    let door_pair = world
        .get::<&Door>(door)
        .map_err(|_| "The door ceased being a door.".to_string())?
        .door_pair;

    let block_type = *world
        .get::<&BlockType>(door_pair)
        .map_err(|_| "The door does not seem to be stuck.".to_string())?;

    match block_type.try_force(world, performer, &performer_name) {
        Err(message) => {
            on_door_failure(world, performer, door, block_type);
            Err(message)
        }
        Ok(tool) => {
            world.remove_one::<BlockType>(door_pair).unwrap();
            if tool == Tool::Blowtorch {
                world
                    .query::<(&Door, &mut TextureType)>()
                    .iter()
                    .filter(|(_, (door, _))| door.door_pair == door_pair)
                    .for_each(|(_, (_, texture_type))| set_is_cut(texture_type));
            }

            action::ok(tool.into_message(&performer_name))
        }
    }
}

fn set_is_cut(texture_type: &mut TextureType) {
    if *texture_type == TextureType::Door {
        *texture_type = TextureType::CutDoor;
    } else if *texture_type == TextureType::Shack {
        *texture_type = TextureType::CutShack;
    }
}

fn on_door_failure(world: &mut World, performer: Entity, door: Entity, block_type: BlockType) {
    let area = world.get::<&Pos>(performer).unwrap().get_area();
    if !core::is_safe(world, performer) {
        return;
    }

    let crew_member = world
        .query::<&Pos>()
        .with::<&CrewMember>()
        .iter()
        .find(|&(crew_member, pos)| {
            crew_member != performer
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
