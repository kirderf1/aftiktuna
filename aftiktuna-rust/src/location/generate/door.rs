use super::Builder;
use crate::asset::location::{DoorPairData, DoorSpawnData};
use crate::core::display::{ModelId, OrderWeight, Symbol};
use crate::core::name::Noun;
use crate::core::position::Pos;
use crate::core::{Door, DoorKind, IsCut};
use hecs::{Entity, World};
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
    let door = world.spawn((
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
    ));
    if is_cut {
        world.insert_one(door, IsCut).unwrap();
    }
    door
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
                DoorPairStatus::Placed => {}
                _ => return Err(format!("Door pair was not fully placed: {pair_id}")),
            }
        }
        Ok(())
    }
}

pub(super) fn place(
    spawn_data: &DoorSpawnData,
    pos: Pos,
    symbol: Symbol,
    builder: &mut Builder,
) -> Result<(), String> {
    {
        let DoorSpawnData {
            pair_id,
            display_type,
            adjective,
        } = spawn_data;

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
