use super::Builder;
use crate::asset::location::{DoorPairData, DoorSpawnData};
use crate::core::display::ModelId;
use crate::core::name::{Adjective, NounId};
use crate::core::position::Pos;
use crate::core::{Door, DoorKind, IsCut};
use hecs::{Entity, World};
use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct DoorInfo {
    pub pos: Pos,
    pub model_id: ModelId,
    pub kind: DoorKind,
    pub noun: NounId,
    pub adjective: Option<Adjective>,
}

pub(crate) fn place_pair(
    world: &mut World,
    door1: DoorInfo,
    door2: DoorInfo,
    pair_data: &DoorPairData,
) -> (Entity, Entity) {
    let door_pair = match pair_data.block_type {
        Some(block_type) => world.spawn((block_type,)),
        None => world.spawn(()),
    };
    let dest1 = door2.pos;
    let dest2 = door1.pos;
    (
        spawn(world, door1, dest1, door_pair, pair_data.is_cut),
        spawn(world, door2, dest2, door_pair, pair_data.is_cut),
    )
}

fn spawn(
    world: &mut World,
    info: DoorInfo,
    destination: Pos,
    door_pair: Entity,
    is_cut: bool,
) -> Entity {
    let mut builder = hecs::EntityBuilder::new();
    builder.add_bundle((
        info.model_id,
        info.noun,
        info.pos,
        Door {
            kind: info.kind,
            destination,
            door_pair,
        },
    ));
    if let Some(adjective) = info.adjective {
        builder.add(adjective);
    }
    if is_cut {
        builder.add(IsCut);
    }
    world.spawn(builder.build())
}

enum DoorPairStatus {
    None,
    One(DoorInfo),
    Placed,
}

pub(super) struct DoorPairsBuilder(HashMap<String, (DoorPairData, DoorPairStatus)>);

impl DoorPairsBuilder {
    pub(super) fn init(door_pairs: super::DoorPairMap) -> Self {
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
                place_pair(world, door_info, other_door.clone(), data);
                DoorPairStatus::Placed
            }
            DoorPairStatus::Placed => {
                return Err(format!("Doors for \"{pair_id}\" placed more than twice"));
            }
        };
        Ok(())
    }

    pub(super) fn verify_all_doors_placed(&self) -> Result<(), String> {
        for (pair_id, (_, status)) in &self.0 {
            match status {
                DoorPairStatus::Placed | DoorPairStatus::None => {}
                _ => return Err(format!("Door pair was not fully placed: {pair_id}")),
            }
        }
        Ok(())
    }
}

pub(super) fn place(
    spawn_data: &DoorSpawnData,
    pos: Pos,
    builder: &mut Builder,
) -> Result<(), String> {
    {
        let DoorSpawnData {
            pair_id,
            door_type,
            model,
            adjective,
        } = spawn_data;

        let door_info = DoorInfo {
            pos,
            model_id: model.clone().unwrap_or_else(|| ModelId::from(*door_type)),
            kind: DoorKind::from(*door_type),
            noun: door_type.noun_id(),
            adjective: adjective.map(|adjective| Adjective(adjective.word().to_owned())),
        };

        builder
            .door_pair_builder
            .add_door(pair_id, door_info, &mut builder.gen_context.world)
    }
}
