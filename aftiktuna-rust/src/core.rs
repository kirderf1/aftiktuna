pub mod area;
pub(crate) mod behavior;
pub(crate) mod combat;
pub(crate) mod inventory;
pub mod item;
pub mod name;
pub mod position;
pub mod status;

pub mod display {
    use serde::{Deserialize, Serialize};
    use std::collections::HashSet;
    use std::path::Path;

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
            Self::new("aftik_portrait")
        }

        pub fn fortuna_chest() -> Self {
            Self::new("container/fortuna_chest")
        }

        pub fn ship() -> Self {
            Self::new("ship")
        }

        pub fn ship_controls() -> Self {
            Self::new("ship_controls")
        }

        pub fn new(name: &str) -> Self {
            Self(name.to_owned())
        }

        pub fn item(name: &str) -> Self {
            Self(format!("item/{name}"))
        }

        pub fn path(&self) -> &str {
            &self.0
        }

        pub fn file_path(&self) -> impl AsRef<Path> + use<> {
            let Self(path) = self;
            format!("assets/texture/object/{path}.json")
        }
    }

    impl Default for ModelId {
        fn default() -> Self {
            Self::unknown()
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct AftikColorId(pub String);

    impl AftikColorId {
        pub fn new(name: &str) -> Self {
            AftikColorId(name.to_owned())
        }
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum DialogueExpression {
        #[default]
        Neutral,
        Excited,
        Sad,
    }

    impl DialogueExpression {
        pub fn variants() -> &'static [Self] {
            use DialogueExpression::*;
            &[Neutral, Excited, Sad]
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum CreatureVariant {
        Female,
        Male,
    }

    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    pub struct CreatureVariantSet(pub HashSet<CreatureVariant>);
}

pub mod store {
    use crate::game_loop::GameState;
    use crate::view;
    use hecs::{Entity, Ref, World};
    use serde::{Deserialize, Serialize};
    use std::fmt::Display;

    use super::item;

    #[derive(Serialize, Deserialize)]
    pub struct Points(pub i32);

    #[derive(Serialize, Deserialize)]
    pub(crate) struct Shopkeeper(pub Vec<StoreStock>);

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
    pub(crate) struct StoreStock {
        pub item: item::ItemTypeId,
        pub price: item::Price,
        pub quantity: StockQuantity,
    }

    #[derive(Serialize, Deserialize)]
    pub struct IsTrading(pub Entity);

    pub(crate) fn get_shop_info(world: &World, character: Entity) -> Option<Ref<'_, Shopkeeper>> {
        let shopkeeper = world.get::<&IsTrading>(character).ok()?.0;
        world.get::<&Shopkeeper>(shopkeeper).ok()
    }

    pub(crate) fn initiate_trade(
        character: Entity,
        shopkeeper: Entity,
        state: &mut GameState,
        view_buffer: &mut view::Buffer,
    ) {
        state
            .world
            .insert_one(character, IsTrading(shopkeeper))
            .unwrap();

        view_buffer.add_change_message(
            "\"Welcome to the store. What do you want to buy?\"".to_owned(),
            state,
        );
    }
}

use self::behavior::BadlyHurtBehavior;
use self::combat::{AttackSet, UnarmedType, WeaponProperties};
use self::status::Stats;
use hecs::Entity;
use serde::{Deserialize, Serialize};

pub const CREW_SIZE_LIMIT: usize = 3;

#[derive(Debug, Serialize, Deserialize)]
pub struct CrewMember(pub Entity);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Species {
    Aftik,
    Goblin,
    Eyesaur,
    Azureclops,
    Scarvie,
    VoraciousFrog,
    BloodMantis,
}

impl Species {
    pub fn model_id(self) -> display::ModelId {
        let name = match self {
            Self::Aftik => "aftik",
            Self::Goblin => "goblin",
            Self::Eyesaur => "eyesaur",
            Self::Azureclops => "azureclops",
            Self::Scarvie => "scarvie",
            Self::VoraciousFrog => "voracious_frog",
            Self::BloodMantis => "blood_mantis",
        };
        display::ModelId(format!("creature/{name}"))
    }

    pub fn noun_id(self) -> name::NounId {
        match self {
            Self::Aftik => "aftik",
            Self::Goblin => "goblin",
            Self::Eyesaur => "eyesaur",
            Self::Azureclops => "azureclops",
            Self::Scarvie => "scarvie",
            Self::VoraciousFrog => "voracious_frog",
            Self::BloodMantis => "blood_mantis",
        }
        .into()
    }

    pub fn default_stats(self) -> Stats {
        match self {
            Self::Aftik => Stats::new(10, 1, 10, 1),
            Self::Goblin => Stats::new(2, 4, 10, 2),
            Self::Eyesaur => Stats::new(7, 7, 4, 2),
            Self::Azureclops => Stats::new(15, 10, 4, 2),
            Self::Scarvie => Stats::new(3, 2, 8, 1),
            Self::VoraciousFrog => Stats::new(8, 8, 3, 3),
            Self::BloodMantis => Stats::new(15, 5, 10, 5),
        }
    }

    pub fn is_large(self) -> bool {
        matches!(self, Self::VoraciousFrog | Self::BloodMantis)
    }

    pub fn unarmed_type(self) -> UnarmedType {
        match self {
            Self::Aftik => UnarmedType::Scratch,
            Self::Goblin => UnarmedType::Scratch,
            Self::Eyesaur => UnarmedType::Bite,
            Self::Azureclops => UnarmedType::Punch,
            Self::Scarvie => UnarmedType::Bite,
            Self::VoraciousFrog => UnarmedType::Pounce,
            Self::BloodMantis => UnarmedType::Slash,
        }
    }

    pub fn attack_set(self) -> AttackSet {
        match self {
            Self::Aftik => AttackSet::Quick,
            Self::Goblin => AttackSet::Light,
            Self::Eyesaur => AttackSet::Quick,
            Self::Azureclops => AttackSet::Varied,
            Self::Scarvie => AttackSet::Light,
            Self::VoraciousFrog => AttackSet::Slow,
            Self::BloodMantis => AttackSet::Quick,
        }
    }

    pub fn badly_hurt_behavior(self) -> Option<BadlyHurtBehavior> {
        match self {
            Self::Aftik => None,
            Self::Goblin => Some(BadlyHurtBehavior::Fearful),
            Self::Eyesaur => None,
            Self::Azureclops => Some(BadlyHurtBehavior::Determined),
            Self::Scarvie => Some(BadlyHurtBehavior::Fearful),
            Self::VoraciousFrog => None,
            Self::BloodMantis => Some(BadlyHurtBehavior::Determined),
        }
    }

    pub fn unarmed_properties(self) -> WeaponProperties {
        WeaponProperties {
            damage_mod: 2.0,
            attack_set: self.attack_set(),
            stun_attack: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag(String);

pub fn any_alive_with_tag(target_tag: &Tag, world: &hecs::World) -> bool {
    world
        .query::<(&status::Health, &Tag)>()
        .iter()
        .any(|(_, (health, tag))| health.is_alive() && target_tag == tag)
}

pub fn find_one_entity_with_tag(target_tag: &Tag, world: &hecs::World) -> Option<Entity> {
    world
        .query::<&Tag>()
        .iter()
        .find(|&(_, tag)| tag == target_tag)
        .map(|(entity, _)| entity)
}

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
}

impl BlockType {
    pub fn variants() -> &'static [Self] {
        use BlockType::*;
        &[Stuck, Sealed]
    }

    pub fn description(self) -> &'static str {
        match self {
            BlockType::Stuck => "stuck",
            BlockType::Sealed => "sealed shut",
        }
    }

    pub fn usable_tools(self) -> Vec<item::Tool> {
        match self {
            BlockType::Stuck => vec![item::Tool::Crowbar, item::Tool::Blowtorch],
            BlockType::Sealed => vec![item::Tool::Blowtorch],
        }
    }
}
