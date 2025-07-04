use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

use crate::action::Action;

pub mod area;
pub mod inventory;
pub mod item;
pub mod name;
pub mod position;
pub mod status;

pub mod display {
    use std::path::Path;

    use serde::{Deserialize, Serialize};

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

        pub fn fortuna_chest() -> Self {
            Self::new("fortuna_chest")
        }

        pub fn ship() -> Self {
            Self::new("ship")
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

        pub fn file_path(&self) -> impl AsRef<Path> {
            let Self(path) = self;
            format!("assets/texture/object/{path}.json")
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

    #[derive(
        Copy, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize,
    )]
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
}

pub mod store {
    use std::fmt::Display;

    use hecs::Entity;
    use serde::{Deserialize, Serialize};

    use super::item;

    #[derive(Serialize, Deserialize)]
    pub struct Points(pub i32);

    #[derive(Serialize, Deserialize)]
    pub struct Shopkeeper(pub Vec<StoreStock>);

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum StockQuantity {
        Unlimited,
        Count(u16),
    }

    impl StockQuantity {
        pub fn is_zero(&self) -> bool {
            matches!(self, Self::Count(0))
        }

        pub fn subtracted(&self, subtracted: u16) -> Option<Self> {
            match self {
                Self::Unlimited => Some(Self::Unlimited),
                Self::Count(count) => Some(Self::Count(count.checked_sub(subtracted)?)),
            }
        }
    }

    impl Display for StockQuantity {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Unlimited => "Unlimited".fmt(f),
                Self::Count(0) => "SOLD OUT".fmt(f),
                Self::Count(count) => count.fmt(f),
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StoreStock {
        pub item: item::Type,
        pub price: item::Price,
        pub quantity: StockQuantity,
    }

    #[derive(Serialize, Deserialize)]
    pub struct IsTrading(pub Entity);
}

pub const CREW_SIZE_LIMIT: usize = 3;

#[derive(Debug, Serialize, Deserialize)]
pub struct CrewMember(pub Entity);

#[derive(Debug, Serialize, Deserialize)]
pub struct Hostile {
    pub aggressive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreatureAttribute {
    Muscular,
    Bulky,
    Agile,
}

impl CreatureAttribute {
    pub fn variants() -> &'static [Self] {
        use CreatureAttribute::*;
        &[Muscular, Bulky, Agile]
    }

    pub fn adjust_stats(self, stats: &mut status::Stats) {
        match self {
            CreatureAttribute::Muscular => {
                stats.strength += 2;
                stats.luck -= 1;
            }
            CreatureAttribute::Bulky => {
                stats.endurance += 2;
                stats.agility -= 1;
            }
            CreatureAttribute::Agile => {
                stats.agility += 2;
                stats.endurance -= 1;
            }
        }
    }

    pub fn as_adjective(self) -> &'static str {
        match self {
            CreatureAttribute::Muscular => "muscular",
            CreatureAttribute::Bulky => "bulky",
            CreatureAttribute::Agile => "agile",
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Waiting;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recruitable;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GivesHuntReward {
    pub target_tag: Tag,
    pub task_message: String,
    pub reward_message: String,
    pub reward: Reward,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reward {
    #[serde(default, skip_serializing_if = "crate::is_default")]
    points: i32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    items: Vec<item::Type>,
}

impl Reward {
    pub fn give_reward_to(&self, target: Entity, world: &mut World) {
        if self.points != 0 {
            let mut crew_points = world
                .get::<&CrewMember>(target)
                .and_then(|crew_member| world.get::<&mut store::Points>(crew_member.0))
                .unwrap();
            crew_points.0 += self.points;
        }

        for item_type in &self.items {
            item_type.spawn(world, inventory::Held::in_inventory(target));
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag(String);

#[derive(Serialize, Deserialize)]
pub struct FortunaChest;

#[derive(Serialize, Deserialize)]
pub struct OpenedChest;

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

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum RepeatingAction {
    TakeAll,
    Rest,
    GoToShip,
}

impl From<RepeatingAction> for Action {
    fn from(value: RepeatingAction) -> Self {
        match value {
            RepeatingAction::TakeAll => Action::TakeAll,
            RepeatingAction::Rest => Action::Rest(false),
            RepeatingAction::GoToShip => Action::GoToShip,
        }
    }
}

pub fn is_safe(world: &World, area: Entity) -> bool {
    world
        .query::<&position::Pos>()
        .with::<&Hostile>()
        .iter()
        .all(|(entity, pos)| !pos.is_in(area) || !status::is_alive(entity, world))
}

pub fn get_wielded_weapon_modifier(world: &World, attacker: Entity) -> f32 {
    inventory::get_wielded(world, attacker)
        .and_then(|item| world.get::<&item::Weapon>(item).map(|weapon| weapon.0).ok())
        .unwrap_or(2.0)
}

pub fn trigger_aggression_in_area(world: &mut World, area: Entity) {
    for (_, (pos, hostile)) in world.query_mut::<(&position::Pos, &mut Hostile)>() {
        if pos.is_in(area) {
            hostile.aggressive = true;
        }
    }
}
