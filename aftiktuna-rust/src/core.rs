use hecs::{CommandBuffer, Entity, Or, World};
use serde::{Deserialize, Serialize};

pub mod area;
pub mod inventory;
pub mod item;
pub mod position;
pub mod status;

#[derive(Debug, Serialize, Deserialize)]
pub struct CrewMember(pub Entity);

#[derive(Debug, Serialize, Deserialize)]
pub struct Aggressive;

#[derive(Debug, Serialize, Deserialize)]
pub struct Threatening;

#[derive(Serialize, Deserialize)]
pub struct Waiting;

#[derive(Serialize, Deserialize)]
pub struct Recruitable;

#[derive(Serialize, Deserialize)]
pub struct FortunaChest;

#[derive(Serialize, Deserialize)]
pub struct OpenedChest;

#[derive(Serialize, Deserialize)]
pub struct Points(pub i32);

#[derive(Serialize, Deserialize)]
pub struct Shopkeeper(pub Vec<PricedItem>);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PricedItem {
    pub item: item::Type,
    pub price: i32,
}

#[derive(Serialize, Deserialize)]
pub struct IsTrading(pub Entity);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Door {
    pub kind: DoorKind,
    pub destination: position::Pos,
    pub door_pair: Entity,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum DoorKind {
    Door,
    Path,
}

#[derive(Serialize, Deserialize)]
pub struct IsCut;

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

    pub fn usable_tools(self) -> Vec<item::Tool> {
        match self {
            BlockType::Stuck => vec![item::Tool::Crowbar, item::Tool::Blowtorch],
            BlockType::Sealed | BlockType::Locked => vec![item::Tool::Blowtorch],
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GoingToShip;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum StopType {
    Win,
    Lose,
}

pub fn is_safe(world: &World, area: Entity) -> bool {
    world
        .query::<&position::Pos>()
        .with::<Or<&Aggressive, &Threatening>>()
        .iter()
        .all(|(entity, pos)| !pos.is_in(area) || !status::is_alive(entity, world))
}

pub fn get_wielded_weapon_modifier(world: &World, attacker: Entity) -> f32 {
    inventory::get_wielded(world, attacker)
        .and_then(|item| world.get::<&item::Weapon>(item).map(|weapon| weapon.0).ok())
        .unwrap_or(2.0)
}

pub fn trigger_aggression_in_area(world: &mut World, area: Entity) {
    let mut buffer = CommandBuffer::new();
    for entity in world
        .query::<&position::Pos>()
        .with::<&Threatening>()
        .iter()
        .filter(|(_, pos)| pos.is_in(area))
        .map(|(entity, _)| entity)
    {
        buffer.remove_one::<Threatening>(entity);
        buffer.insert_one(entity, Aggressive);
    }
    buffer.run_on(world);
}
