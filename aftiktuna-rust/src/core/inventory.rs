use crate::core::item::Tool;
use crate::core::position::Pos;
use hecs::{Entity, Query, World};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Held {
    holder: Entity,
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

pub fn is_holding<Q: Query>(world: &World, holder: Entity) -> bool {
    world
        .query::<&Held>()
        .with::<Q>()
        .iter()
        .any(|(_, held)| held.held_by(holder))
}

pub fn is_holding_tool(world: &World, holder: Entity, requested_tool: Tool) -> bool {
    world
        .query::<(&Held, &Tool)>()
        .iter()
        .any(|(_, (held, &tool))| held.held_by(holder) && tool == requested_tool)
}

pub fn is_in_inventory(world: &World, item: Entity, holder: Entity) -> bool {
    world
        .get::<&Held>(item)
        .ok()
        .map_or(false, |held| held.is_in_inventory(holder))
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

pub fn consume_one<Q: Query>(world: &mut World, holder: Entity) -> Option<()> {
    let (item, _) = world
        .query::<&Held>()
        .with::<Q>()
        .iter()
        .find(|(_, held)| held.held_by(holder))?;
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
