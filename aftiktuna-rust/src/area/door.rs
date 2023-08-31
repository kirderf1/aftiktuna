use crate::action::door::{BlockType, Door, DoorKind};
use crate::core::position::Pos;
use crate::view::name::Noun;
use crate::view::{DisplayInfo, OrderWeight, TextureType};
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct DoorInfo {
    pub pos: Pos,
    pub symbol: char,
    pub texture_type: TextureType,
    pub kind: DoorKind,
    pub name: Noun,
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
        DisplayInfo::new(info.symbol, info.texture_type),
        OrderWeight::Background,
        info.name,
        info.pos,
        Door {
            kind: info.kind,
            destination,
            door_pair,
        },
    ))
}

#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoorType {
    Door,
    Shack,
    House,
    Store,
    Path,
}

impl From<DoorType> for TextureType {
    fn from(value: DoorType) -> Self {
        match value {
            DoorType::Door => TextureType::Door,
            DoorType::Shack | DoorType::House | DoorType::Store => TextureType::Shack,
            DoorType::Path => TextureType::Path,
        }
    }
}

impl From<DoorType> for DoorKind {
    fn from(value: DoorType) -> Self {
        match value {
            DoorType::Door | DoorType::Shack | DoorType::House | DoorType::Store => DoorKind::Door,
            DoorType::Path => DoorKind::Path,
        }
    }
}

impl DoorType {
    pub fn noun(self, adjective: Option<Adjective>) -> Noun {
        let noun = match self {
            DoorType::Door => Noun::new("door", "doors"),
            DoorType::Shack => Noun::new("shack", "shacks"),
            DoorType::House => Noun::new("house", "houses"),
            DoorType::Store => Noun::new("store", "stores"),
            DoorType::Path => Noun::new("path", "paths"),
        };
        if let Some(adjective) = adjective {
            noun.with_adjective(adjective.word())
        } else {
            noun
        }
    }
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
