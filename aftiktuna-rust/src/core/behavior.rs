use super::display::DialogueExpression;
use super::inventory::Held;
use super::item::ItemType;
use super::position::Pos;
use super::{CrewMember, Tag, status, store};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Character;

#[derive(Debug, Serialize, Deserialize)]
pub struct Hostile {
    pub aggressive: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Wandering;

#[derive(Debug, Serialize, Deserialize)]
pub struct ObservationTarget;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadlyHurtBehavior {
    Fearful,
    Determined,
}

#[derive(Serialize, Deserialize)]
pub enum Intention {
    Wield(hecs::Entity),
    Force(hecs::Entity),
    UseMedkit(hecs::Entity),
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum RepeatingAction {
    TakeAll,
    Rest,
    GoToShip,
    ChargedAttack(hecs::Entity),
}

impl RepeatingAction {
    pub fn cancel_if_unsafe(self) -> bool {
        !matches!(self, Self::ChargedAttack(_))
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GivesHuntReward {
    pub target_tag: Tag,
    pub task_dialogue: DialogueNode,
    pub reward_dialogue: DialogueNode,
    pub reward: Reward,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueNode {
    pub expression: DialogueExpression,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reward {
    #[serde(default, skip_serializing_if = "crate::is_default")]
    points: i32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    items: Vec<ItemType>,
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
