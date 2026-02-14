use super::inventory::Held;
use super::item::ItemTypeId;
use super::position::Pos;
use super::{CrewMember, DialogueId, Tag, status, store};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Character;

#[derive(Debug, Serialize, Deserialize)]
pub struct Hostile {
    pub aggressive: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Wandering {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area_tag: Option<Tag>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObservationTarget;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BadlyHurtBehavior {
    /// With this behavior, the entity will flee when hurt.
    Fearful,
    /// With this behavior, the entity will prioritize rash attacks when hurt.
    Determined,
}

#[derive(Serialize, Deserialize)]
pub enum Intention {
    Wield(hecs::Entity),
    Force {
        door: hecs::Entity,
        assisted: hecs::Entity,
    },
    UseMedkit(hecs::Entity),
    Refuel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackgroundId(String);

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum RepeatingAction {
    TakeAll,
    Rest,
    GoToShip,
    UseItem { item: hecs::Entity, use_time: u16 },
    ChargedAttack(hecs::Entity),
}

impl RepeatingAction {
    pub fn cancel_if_unsafe(self) -> bool {
        !matches!(self, Self::ChargedAttack(_)) && !matches!(self, Self::UseItem { .. })
    }
}

#[derive(Serialize, Deserialize)]
pub struct Waiting {
    pub at_ship: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrewLossMemory {
    pub name: String,
    pub recent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recruitable;

/// Dialogue that appears after the greeting as a response to the talk action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Talk(pub DialogueId);

#[derive(Clone, Serialize, Deserialize)]
pub struct GivesHuntRewardData {
    pub target_tag: Tag,
    pub target_label: String,
    pub task_dialogue: DialogueId,
    pub already_completed_dialogue: DialogueId,
    pub reward_dialogue: DialogueId,
    pub reward: Reward,
    pub presented: bool,
}

impl GivesHuntRewardData {
    pub fn is_fulfilled(&self, world: &hecs::World) -> bool {
        !super::any_alive_with_tag(&self.target_tag, world)
    }
}

/// Dialogue between npcs triggered by encounter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundDialogue {
    /// The tag is expected to only match one other entity.
    pub target: Tag,
    pub dialogue: DialogueId,
}

/// Dialogue towards the player triggered by encounter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterDialogue(pub DialogueId);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reward {
    #[serde(default, skip_serializing_if = "crate::is_default")]
    points: i32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    items: Vec<ItemTypeId>,
}

impl Reward {
    pub fn give_reward_to(&self, target: hecs::Entity, world: &mut hecs::World) {
        if self.points != 0 {
            let mut crew_points = world
                .get::<&CrewMember>(target)
                .and_then(|crew_member| world.get::<&mut store::Points>(crew_member.0))
                .unwrap();
            crew_points.0 += self.points;
        }

        for item_type in &self.items {
            item_type.spawn(world, Held::in_inventory(target));
        }
    }
}

/// Assigned to crew entity when any crew member have brought up that the crew has sufficient fuel to leave.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TalkedAboutEnoughFuel;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TalkState {
    pub talked_about_badly_hurt: bool,
}

pub fn is_safe(world: &hecs::World, area: hecs::Entity) -> bool {
    world
        .query::<&Pos>()
        .with::<&Hostile>()
        .iter()
        .all(|(entity, pos)| !pos.is_in(area) || !status::is_alive(entity, world))
}

pub fn trigger_aggression_in_area(world: &mut hecs::World, area: hecs::Entity) {
    for (_, (pos, hostile)) in world.query_mut::<(&Pos, &mut Hostile)>() {
        if pos.is_in(area) {
            hostile.aggressive = true;
        }
    }
}
