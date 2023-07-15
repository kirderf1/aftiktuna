use crate::action::door::{BlockType, Door};
use crate::position::Pos;
use crate::view::{DisplayInfo, NameData, NounData, TextureType};
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct DoorInfo {
    pub pos: Pos,
    pub symbol: char,
    pub texture_type: TextureType,
    pub name: NameData,
}

pub fn place_pair(
    world: &mut World,
    door1: DoorInfo,
    door2: DoorInfo,
    block_type: Option<BlockType>,
) {
    let door_pair = match block_type {
        Some(block_type) => world.spawn((block_type,)),
        None => world.spawn(()),
    };
    let dest1 = door2.pos;
    let dest2 = door1.pos;
    place(world, door1, dest1, door_pair);
    place(world, door2, dest2, door_pair);
}

fn place(world: &mut World, info: DoorInfo, destination: Pos, door_pair: Entity) -> Entity {
    world.spawn((
        DisplayInfo::new(info.symbol, info.texture_type, 20),
        info.name,
        info.pos,
        Door {
            destination,
            door_pair,
        },
    ))
}

#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoorType {
    Door,
    Path,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Adjective {
    Left,
    Middle,
    Right,
}

impl Adjective {
    fn word(self) -> &'static str {
        match self {
            Adjective::Left => "left",
            Adjective::Middle => "middle",
            Adjective::Right => "right",
        }
    }
}

impl DoorType {
    pub fn name_data(self, adjective: Option<Adjective>) -> NameData {
        let mut noun = match self {
            DoorType::Door => NounData::new("door", "doors"),
            DoorType::Path => NounData::new("path", "paths"),
        };
        if let Some(adjective) = adjective {
            noun = noun.with_adjective(adjective.word());
        }
        NameData::Noun(noun)
    }
    pub fn texture_type(self) -> TextureType {
        match self {
            DoorType::Door => TextureType::Door,
            DoorType::Path => TextureType::Path,
        }
    }
}
