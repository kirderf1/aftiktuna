use crate::core::item::{ItemTypeId, Tool};
use crate::core::position::Pos;
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Held {
    pub holder: Entity,
    in_hand: bool,
}

impl Held {
    pub fn held_by(&self, holder: Entity) -> bool {
        self.holder == holder
    }

    pub fn is_in_hand(&self) -> bool {
        self.in_hand
    }

    pub fn is_in_inventory(&self, holder: Entity) -> bool {
        self.held_by(holder) && !self.in_hand
    }

    pub fn in_inventory(holder: Entity) -> Self {
        Self {
            holder,
            in_hand: false,
        }
    }

    pub fn in_hand(holder: Entity) -> Self {
        Self {
            holder,
            in_hand: true,
        }
    }
}

pub fn is_holding(
    item_type_matcher: impl Fn(&ItemTypeId) -> bool,
    world: &World,
    holder: Entity,
) -> bool {
    world
        .query::<(&ItemTypeId, &Held)>()
        .iter()
        .any(|(_, (item_type, held))| held.held_by(holder) && item_type_matcher(item_type))
}

pub fn is_holding_tool(world: &World, holder: Entity, requested_tool: Tool) -> bool {
    world
        .query::<(&ItemTypeId, &Held)>()
        .iter()
        .any(|(_, (item_type, held))| held.held_by(holder) && requested_tool.matches(item_type))
}

pub fn is_in_inventory(world: &World, item: Entity, holder: Entity) -> bool {
    world
        .get::<&Held>(item)
        .ok()
        .is_some_and(|held| held.is_in_inventory(holder))
}

pub fn get_held(world: &World, holder: Entity) -> Vec<Entity> {
    world
        .query::<&Held>()
        .iter()
        .filter(|(_, held)| held.held_by(holder))
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>()
}

pub fn get_inventory(world: &World, holder: Entity) -> Vec<Entity> {
    world
        .query::<&Held>()
        .iter()
        .filter(|(_, held)| held.is_in_inventory(holder))
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>()
}

pub fn get_wielded(world: &World, holder: Entity) -> Option<Entity> {
    world
        .query::<&Held>()
        .iter()
        .find(|(_, held)| held.held_by(holder) && held.in_hand)
        .map(|(item, _)| item)
}

pub fn consume_one(
    item_type_matcher: impl Fn(&ItemTypeId) -> bool,
    world: &mut World,
    holder: Entity,
) -> Option<()> {
    let (item, _) = world
        .query::<(&ItemTypeId, &Held)>()
        .iter()
        .find(|&(_, (item_type, held))| held.held_by(holder) && item_type_matcher(item_type))?;
    world.despawn(item).ok()
}

pub fn unwield_if_needed(world: &mut World, holder: Entity) {
    world
        .query_mut::<&mut Held>()
        .into_iter()
        .filter(|(_, held)| held.held_by(holder))
        .for_each(|(_, held)| held.in_hand = false);
}

pub fn drop_all_items(world: &mut World, entity: Entity) {
    let pos = *world.get::<&Pos>(entity).unwrap();
    let items = get_held(world, entity);
    for item in items {
        world.exchange_one::<Held, _>(item, pos).unwrap();
    }
}

pub fn fuel_cans_held_by_crew(world: &World, excluding_items: &[Entity]) -> usize {
    world
        .query::<(&ItemTypeId, &Held)>()
        .iter()
        .filter(|&(item, (item_type, held))| {
            item_type.is_fuel_can()
                && !excluding_items.contains(&item)
                && world
                    .satisfies::<&super::CrewMember>(held.holder)
                    .unwrap_or(false)
        })
        .count()
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Container;
