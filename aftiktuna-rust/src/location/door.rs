use crate::core::display::{ModelId, OrderWeight, Symbol};
use crate::core::name::Noun;
use crate::core::position::Pos;
use crate::core::{BlockType, Door, DoorKind};
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct DoorInfo {
    pub pos: Pos,
    pub symbol: Symbol,
    pub model_id: ModelId,
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
        info.symbol,
        info.model_id,
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
    LeftPath,
    RightPath,
    CrossroadPath,
}

impl From<DoorType> for ModelId {
    fn from(value: DoorType) -> Self {
        match value {
            DoorType::Door => ModelId::new("door"),
            DoorType::Shack | DoorType::House | DoorType::Store => ModelId::new("shack"),
            DoorType::Path => ModelId::new("path"),
            DoorType::LeftPath => ModelId::new("path/left_corner"),
            DoorType::RightPath => ModelId::new("path/right_corner"),
            DoorType::CrossroadPath => ModelId::new("path/crossroad"),
        }
    }
}

impl From<DoorType> for DoorKind {
    fn from(value: DoorType) -> Self {
        match value {
            DoorType::Door | DoorType::Shack | DoorType::House | DoorType::Store => DoorKind::Door,
            DoorType::Path | DoorType::LeftPath | DoorType::RightPath | DoorType::CrossroadPath => {
                DoorKind::Path
            }
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
            DoorType::Path | DoorType::LeftPath | DoorType::RightPath | DoorType::CrossroadPath => {
                Noun::new("path", "paths")
            }
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
