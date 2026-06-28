use crate::OneOrList;
use crate::core::behavior::{self, CrewLossMemory, Passenger};
use crate::core::display::DialogueExpression;
use crate::core::name::Name;
use crate::core::position::Pos;
use crate::core::status::{Health, Morale, MoraleState};
use crate::core::{DialogueId, area, inventory};
use crate::game_loop::GameState;
use hecs::Entity;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DialogueCondition {
    IsBadlyHurt(bool),
    TargetIsBadlyHurt(bool),
    HasEnoughFuel(bool),
    AtShip(bool),
    AtFortuna(bool),
    IsPassenger(bool),
    HasKnownName(bool),
    HasCrewLossMemory(bool),
    HasRecentCrewLossMemory(bool),
    HasBackground(behavior::BackgroundId),
    MoraleIsAtLeast(MoraleState),
}

impl DialogueCondition {
    fn test(&self, speaker: Entity, target: Entity, state: &GameState) -> bool {
        let world = &state.world;
        match self {
            &Self::IsBadlyHurt(is_badly_hurt) => {
                is_badly_hurt
                    == world
                        .get::<&Health>(speaker)
                        .is_ok_and(|health| health.is_badly_hurt())
            }
            &Self::TargetIsBadlyHurt(target_is_badly_hurt) => {
                target_is_badly_hurt
                    == world
                        .get::<&Health>(target)
                        .is_ok_and(|health| health.is_badly_hurt())
            }
            &Self::HasEnoughFuel(has_enough_fuel) => {
                has_enough_fuel
                    == area::fuel_needed_to_launch(world).is_some_and(|fuel_amount| {
                        fuel_amount <= inventory::fuel_cans_held_by_crew(world, &[])
                    })
            }
            &Self::AtShip(at_ship) => {
                at_ship
                    == world
                        .get::<&Pos>(speaker)
                        .is_ok_and(|pos| area::is_in_ship(*pos, world))
            }
            &Self::AtFortuna(at_fortuna) => at_fortuna == state.generation_state.is_at_fortuna(),
            &Self::IsPassenger(is_passenger) => {
                is_passenger == world.satisfies::<&Passenger>(speaker)
            }
            &Self::HasKnownName(has_known_name) => world
                .get::<&Name>(speaker)
                .is_ok_and(|name| has_known_name == name.is_known),
            &Self::HasCrewLossMemory(has_crew_loss_memory) => {
                has_crew_loss_memory == world.satisfies::<&CrewLossMemory>(speaker)
            }
            &Self::HasRecentCrewLossMemory(has_recent_crew_loss_memory) => {
                has_recent_crew_loss_memory
                    == world
                        .get::<&CrewLossMemory>(speaker)
                        .is_ok_and(|crew_loss_memory| crew_loss_memory.recent)
            }
            Self::HasBackground(expected_background) => world
                .get::<&behavior::BackgroundId>(speaker)
                .is_ok_and(|checked_background| *checked_background == *expected_background),
            &Self::MoraleIsAtLeast(morale_state) => {
                morale_state
                    <= world
                        .get::<&Morale>(speaker)
                        .map(|morale| morale.state())
                        .unwrap_or_default()
            }
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(
    from = "OneOrList<DialogueCondition>",
    into = "OneOrList<DialogueCondition>"
)]
pub struct DialogueConditionList(pub Vec<DialogueCondition>);

impl DialogueConditionList {
    pub fn test(&self, speaker: Entity, target: Entity, state: &GameState) -> bool {
        self.0
            .iter()
            .all(|condition| condition.test(speaker, target, state))
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<OneOrList<DialogueCondition>> for DialogueConditionList {
    fn from(value: OneOrList<DialogueCondition>) -> Self {
        match value {
            OneOrList::One(condition) => Self(vec![condition]),
            OneOrList::List(list) => Self(list),
        }
    }
}

impl From<DialogueConditionList> for OneOrList<DialogueCondition> {
    fn from(value: DialogueConditionList) -> Self {
        if value.0.len() == 1 {
            Self::One(value.0.into_iter().next().unwrap())
        } else {
            Self::List(value.0)
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConditionedDialogueNode {
    #[serde(default, skip_serializing_if = "DialogueConditionList::is_empty")]
    pub condition: DialogueConditionList,
    pub expression: DialogueExpression,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply: Option<DialogueList>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DialogueList(Vec<ConditionedDialogueNode>);

impl DialogueList {
    pub(crate) fn select_node(
        &self,
        speaker: Entity,
        target: Entity,
        state: &GameState,
    ) -> Option<&ConditionedDialogueNode> {
        self.0
            .iter()
            .find(|node| node.condition.test(speaker, target, state))
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DialogueEffect {
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub set_talked_about_enough_fuel: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NextDialogueKind {
    Response,
    Continuation,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NextDialogueData {
    pub kind: NextDialogueKind,
    #[serde(default, skip_serializing_if = "DialogueConditionList::is_empty")]
    pub condition: DialogueConditionList,
    pub node: RefOrData,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DialogueData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect: Option<DialogueEffect>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default, skip_serializing_if = "DialogueList::is_empty")]
    pub dialogue: DialogueList,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub next: Vec<NextDialogueData>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RefOrData {
    Ref(DialogueId),
    Data(DialogueData),
}

pub fn load_dialogue_data(name: &str) -> Result<DialogueData, super::Error> {
    super::load_json_asset(format!("dialogue/{name}.json"))
}
