use hecs::{CommandBuffer, Entity, Or, World};
use serde::{Deserialize, Serialize};

pub mod area;
pub mod inventory;
pub mod item;
pub mod name;
pub mod position;
pub mod status;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ModelId(pub String);

impl ModelId {
    pub fn unknown() -> Self {
        Self::new("unknown")
    }
    pub fn small_unknown() -> Self {
        Self::new("small_unknown")
    }

    pub fn portrait() -> Self {
        Self::new("portrait")
    }

    pub fn aftik() -> Self {
        Self::creature("aftik")
    }

    pub fn new(name: &str) -> Self {
        Self(name.to_owned())
    }

    pub fn item(name: &str) -> Self {
        Self(format!("item/{name}"))
    }

    pub fn creature(name: &str) -> Self {
        Self(format!("creature/{name}"))
    }

    pub fn path(&self) -> &str {
        &self.0
    }
}

impl Default for ModelId {
    fn default() -> Self {
        Self::unknown()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol(pub char);

impl Symbol {
    pub fn from_name(name: &str) -> Self {
        Self(name.chars().next().unwrap().to_ascii_uppercase())
    }
}

#[derive(Copy, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub enum OrderWeight {
    Item,
    Controlled,
    #[default]
    Creature,
    Background,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AftikColorId(pub String);

impl AftikColorId {
    pub fn new(name: &str) -> Self {
        AftikColorId(name.to_owned())
    }
}

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
