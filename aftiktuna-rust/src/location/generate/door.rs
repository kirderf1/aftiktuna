use super::Builder;
use crate::core::display::{ModelId, OrderWeight, Symbol};
use crate::core::name::Noun;
use crate::core::position::Pos;
use crate::core::{BlockType, Door, DoorKind};
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct DoorInfo {
    pub pos: Pos,
    pub symbol: Symbol,
    pub model_id: ModelId,
    pub kind: DoorKind,
    pub name: Noun,
}

pub(crate) fn place_pair(
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoorType {
    Door,
    Doorway,
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
            DoorType::Doorway => ModelId::new("doorway"),
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
            DoorType::Door
            | DoorType::Doorway
            | DoorType::Shack
            | DoorType::House
            | DoorType::Store => DoorKind::Door,
            DoorType::Path | DoorType::LeftPath | DoorType::RightPath | DoorType::CrossroadPath => {
                DoorKind::Path
            }
        }
    }
}

impl DoorType {
    pub fn variants() -> &'static [Self] {
        use DoorType::*;
        &[
            Door,
            Doorway,
            Shack,
            House,
            Store,
            Path,
            LeftPath,
            RightPath,
            CrossroadPath,
        ]
    }

    pub fn noun(self, adjective: Option<Adjective>) -> Noun {
        let noun = match self {
            DoorType::Door => Noun::new("door", "doors"),
            DoorType::Doorway => Noun::new("doorway", "doorways"),
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

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Adjective {
    Left,
    Middle,
    Right,
}

impl Adjective {
    pub fn variants() -> &'static [Self] {
        use Adjective::*;
        &[Left, Middle, Right]
    }

    pub fn word(self) -> &'static str {
        match self {
            Adjective::Left => "left",
            Adjective::Middle => "middle",
            Adjective::Right => "right",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DoorPairData {
    #[serde(default)]
    pub block_type: Option<BlockType>,
}

enum DoorPairStatus {
    None,
    One(DoorInfo),
    Placed,
}

pub(super) struct DoorPairsBuilder(HashMap<String, (DoorPairData, DoorPairStatus)>);

impl DoorPairsBuilder {
    pub(super) fn init(door_pairs: HashMap<String, DoorPairData>) -> Self {
        Self(
            door_pairs
                .into_iter()
                .map(|(key, data)| (key.to_string(), (data, DoorPairStatus::None)))
                .collect(),
        )
    }

    fn add_door(
        &mut self,
        pair_id: &str,
        door_info: DoorInfo,
        world: &mut World,
    ) -> Result<(), String> {
        let (data, status) = self
            .0
            .get_mut(pair_id)
            .ok_or_else(|| format!("Unknown door id \"{pair_id}\""))?;

        *status = match status {
            DoorPairStatus::None => DoorPairStatus::One(door_info),
            DoorPairStatus::One(other_door) => {
                place_pair(world, door_info, other_door.clone(), data.block_type);
                DoorPairStatus::Placed
            }
            DoorPairStatus::Placed => {
                return Err(format!("Doors for \"{pair_id}\" placed more than twice"))
            }
        };
        Ok(())
    }

    pub(super) fn verify_all_doors_placed(&self) -> Result<(), String> {
        for (pair_id, (_, status)) in &self.0 {
            match status {
                DoorPairStatus::Placed => {}
                _ => return Err(format!("Door pair was not fully placed: {pair_id}")),
            }
        }
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DoorSpawnData {
    pub pair_id: String,
    pub display_type: DoorType,
    #[serde(default)]
    pub adjective: Option<Adjective>,
}

impl DoorSpawnData {
    pub(super) fn place(
        &self,
        pos: Pos,
        symbol: Symbol,
        builder: &mut Builder,
    ) -> Result<(), String> {
        {
            let Self {
                pair_id,
                display_type,
                adjective,
            } = self;

            let door_info = DoorInfo {
                pos,
                symbol,
                model_id: ModelId::from(*display_type),
                kind: DoorKind::from(*display_type),
                name: display_type.noun(*adjective),
            };

            builder
                .door_pair_builder
                .add_door(pair_id, door_info, &mut builder.gen_context.world)
        }
    }
}
