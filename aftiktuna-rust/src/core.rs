pub mod area;
pub mod inventory;
pub mod item;
pub(crate) mod name;
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
}

pub mod store {
    use hecs::{Entity, Ref, World};
    use serde::{Deserialize, Serialize};
    use std::fmt::Display;

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
        pub item: item::ItemType,
        pub price: item::Price,
        pub quantity: StockQuantity,
    }

    #[derive(Serialize, Deserialize)]
    pub struct IsTrading(pub Entity);

    pub fn get_shop_info(world: &World, character: Entity) -> Option<Ref<'_, Shopkeeper>> {
        let shopkeeper = world.get::<&IsTrading>(character).ok()?.0;
        world.get::<&Shopkeeper>(shopkeeper).ok()
    }
}

use self::display::DialogueExpression;
use crate::action::Action;
use crate::core::name::Noun;
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub const CREW_SIZE_LIMIT: usize = 3;

#[derive(Debug, Serialize, Deserialize)]
pub struct CrewMember(pub Entity);

#[derive(Debug, Serialize, Deserialize)]
pub struct Character;

#[derive(Debug, Serialize, Deserialize)]
pub struct CrewLossMemory {
    pub name: String,
    pub recent: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Hostile {
    pub aggressive: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Wandering;

#[derive(Debug, Serialize, Deserialize)]
pub struct ObservationTarget;

#[derive(Clone, Copy, Debug)]
pub enum UnarmedType {
    Bite,
    Scratch,
    Punch,
    Pounce,
    Slash,
}

impl UnarmedType {
    pub fn attack_verb(self) -> &'static str {
        match self {
            Self::Bite | Self::Pounce => "jumps at",
            Self::Scratch => "scratches at",
            Self::Punch => "launches a punch at",
            Self::Slash => "slashes at",
        }
    }

    pub fn hit_verb(self) -> &'static str {
        match self {
            Self::Bite => "bites",
            Self::Scratch | Self::Punch | Self::Slash => "hits",
            Self::Pounce => "pounces",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackKind {
    Light,
    Rash,
    Charged,
}

impl AttackKind {
    pub fn hit_modifier(self) -> i16 {
        match self {
            Self::Light => 0,
            Self::Rash => -2,
            Self::Charged => 2,
        }
    }

    pub fn stun_modifier(self) -> i16 {
        match self {
            Self::Light => -3,
            Self::Rash => 3,
            Self::Charged => 6,
        }
    }

    pub fn damage_modifier(self) -> f32 {
        match self {
            Self::Light => 1.,
            Self::Rash => 1.75,
            Self::Charged => 2.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackSet {
    Light,
    Quick,
    Slow,
    Intense,
    Varied,
}

impl AttackSet {
    pub fn available_kinds(self) -> &'static [AttackKind] {
        use AttackKind::*;
        match self {
            AttackSet::Light => &[Light],
            AttackSet::Quick => &[Light, Rash],
            AttackSet::Slow => &[Light, Charged],
            AttackSet::Intense => &[Rash, Charged],
            AttackSet::Varied => &[Light, Rash, Charged],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadlyHurtBehavior {
    Fearful,
    Determined,
}

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

    pub fn noun(self) -> Noun {
        use name::IndefiniteArticle::*;
        match self {
            Self::Aftik => Noun::new("aftik", "aftiks", An),
            Self::Goblin => Noun::new("goblin", "goblins", A),
            Self::Eyesaur => Noun::new("eyesaur", "eyesaurs", An),
            Self::Azureclops => Noun::new("azureclops", "azureclopses", An),
            Self::Scarvie => Noun::new("scarvie", "scarvies", A),
            Self::VoraciousFrog => Noun::new("voracious frog", "voracious frogs", A),
            Self::BloodMantis => Noun::new("blood mantis", "blood mantes", A),
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
                stats.strength += 3;
                stats.luck -= 1;
            }
            CreatureAttribute::Bulky => {
                stats.endurance += 3;
                stats.agility -= 1;
            }
            CreatureAttribute::Agile => {
                stats.agility += 3;
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

impl Display for CreatureAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.as_adjective(), f)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Waiting {
    pub at_ship: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recruitable;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GivesHuntReward {
    pub target_tag: Tag,
    pub task_expression: DialogueExpression,
    pub task_message: String,
    pub reward_expression: DialogueExpression,
    pub reward_message: String,
    pub reward: Reward,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reward {
    #[serde(default, skip_serializing_if = "crate::is_default")]
    points: i32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    items: Vec<item::ItemType>,
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

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum RepeatingAction {
    TakeAll,
    Rest,
    GoToShip,
    ChargedAttack(Entity),
}

impl RepeatingAction {
    pub fn cancel_if_unsafe(self) -> bool {
        !matches!(self, Self::ChargedAttack(_))
    }
}

impl From<RepeatingAction> for Action {
    fn from(value: RepeatingAction) -> Self {
        match value {
            RepeatingAction::TakeAll => Action::TakeAll,
            RepeatingAction::Rest => Action::Rest(false),
            RepeatingAction::GoToShip => Action::GoToShip,
            RepeatingAction::ChargedAttack(target) => Action::ChargedAttack(target),
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

#[derive(Clone, Copy)]
pub struct WeaponProperties {
    pub damage_mod: f32,
    pub attack_set: AttackSet,
    pub stun_attack: bool,
}

pub fn get_active_weapon_properties(world: &World, attacker: Entity) -> WeaponProperties {
    inventory::get_wielded(world, attacker)
        .and_then(|item| {
            world
                .get::<&item::ItemType>(item)
                .ok()
                .and_then(|item_type| item_type.weapon_properties())
        })
        .unwrap_or_else(|| {
            world
                .get::<&Species>(attacker)
                .unwrap()
                .unarmed_properties()
        })
}

pub fn trigger_aggression_in_area(world: &mut World, area: Entity) {
    for (_, (pos, hostile)) in world.query_mut::<(&position::Pos, &mut Hostile)>() {
        if pos.is_in(area) {
            hostile.aggressive = true;
        }
    }
}
