use crate::core::area::ShipControls;
use crate::core::item::{CanWield, Item, Medkit};
use crate::core::{
    inventory, BlockType, CrewMember, Door, FortunaChest, IsFoe, PricedItem, Recruitable,
    Shopkeeper, Waiting,
};
use crate::game_loop::GameState;
use crate::location::Choice;
use crate::view::area::ItemProfile;
use crate::view::name::NameData;
use hecs::Entity;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[derive(Clone, Eq)]
pub enum Suggestion {
    Simple(String),
    Recursive(String, Vec<Suggestion>),
}

impl Suggestion {
    pub fn text(&self) -> &str {
        match self {
            Suggestion::Simple(command) => command,
            Suggestion::Recursive(text, _) => text,
        }
    }

    fn is_empty(&self) -> bool {
        matches!(self, Suggestion::Recursive(_, suggestions) if suggestions.is_empty())
    }

    fn flatten(self) -> Self {
        match self {
            Suggestion::Recursive(_, suggestions) if suggestions.len() == 1 => {
                suggestions.into_iter().next().unwrap()
            }
            suggestion => suggestion,
        }
    }
}

impl PartialEq for Suggestion {
    fn eq(&self, other: &Self) -> bool {
        self.text().eq(other.text())
    }
}

impl Hash for Suggestion {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.text().hash(state);
    }
}

impl PartialOrd for Suggestion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.text().partial_cmp(other.text())
    }
}

impl Ord for Suggestion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.text().cmp(other.text())
    }
}

macro_rules! simple {
    ($($tokens:tt)*) => {
        Suggestion::Simple(format!($($tokens)*))
    };
}

macro_rules! recursive {
    ($elements:expr, $($tokens:tt)*) => {
        Suggestion::Recursive(
            format!($($tokens)*, "_"),
            sorted_without_duplicates($elements.map(|element| simple!($($tokens)*, element))),
        )
    };
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum InteractionType {
    Item,
    Wieldable,
    UseMedkit,
    Door,
    Forceable,
    ShipControls,
    Openable,
    CrewMember,
    Controlled,
    Shopkeeper,
    Recruitable,
    Waiting,
    Following,
    Foe,
}

impl InteractionType {
    pub fn commands(self, name: &str, inventory: &[ItemProfile]) -> Vec<Suggestion> {
        let name = name.to_lowercase();
        match self {
            InteractionType::Item => vec![simple!("take {name}"), simple!("check {name}")],
            InteractionType::Wieldable => vec![simple!("wield {name}")],
            InteractionType::UseMedkit => vec![simple!("use medkit")],
            InteractionType::Door => vec![simple!("enter {name}")],
            InteractionType::Forceable => vec![simple!("force {name}")],
            InteractionType::ShipControls => {
                vec![simple!("launch ship"), simple!("refuel ship")]
            }
            InteractionType::Openable => vec![simple!("open {name}")],
            InteractionType::CrewMember => {
                vec![
                    simple!("control {name}"),
                    simple!("status"),
                    simple!("rest"),
                    simple!("talk to {name}"),
                    simple!("tell {name} to wait at ship"),
                    recursive!(inventory.iter().map(ItemProfile::name), "give {name} {}"),
                ]
            }
            InteractionType::Controlled => {
                vec![
                    simple!("status"),
                    simple!("rest"),
                    simple!("wait"),
                    simple!("go to ship"),
                    recursive!(
                        inventory
                            .iter()
                            .filter(|item| item.is_wieldable && !item.is_wielded)
                            .map(ItemProfile::name),
                        "wield {}"
                    ),
                    recursive!(inventory.iter().map(ItemProfile::name), "check {}"),
                ]
            }
            InteractionType::Shopkeeper => vec![simple!("trade")],
            InteractionType::Recruitable => {
                vec![simple!("recruit {name}"), simple!("talk to {name}")]
            }
            InteractionType::Waiting => vec![simple!("tell {name} to follow")],
            InteractionType::Following => vec![simple!("tell {name} to wait")],
            InteractionType::Foe => vec![simple!("attack {name}")],
        }
    }
}

pub fn interactions_for(entity: Entity, state: &GameState) -> Vec<InteractionType> {
    let mut interactions = Vec::new();
    let world = &state.world;
    let entity_ref = world.entity(entity).unwrap();

    if entity_ref.satisfies::<&Item>() {
        interactions.push(InteractionType::Item);
    }
    if entity_ref.satisfies::<&CanWield>() {
        interactions.push(InteractionType::Wieldable);
    }
    if let Some(door) = entity_ref.get::<&Door>() {
        interactions.push(InteractionType::Door);
        if world.get::<&BlockType>(door.door_pair).is_ok() {
            interactions.push(InteractionType::Forceable);
        }
    }
    if entity_ref.satisfies::<&FortunaChest>() {
        interactions.push(InteractionType::Openable);
    }
    if entity != state.controlled && entity_ref.satisfies::<&CrewMember>() {
        interactions.push(InteractionType::CrewMember);
        if entity_ref.satisfies::<&Waiting>() {
            interactions.push(InteractionType::Waiting);
        } else {
            interactions.push(InteractionType::Following);
        }
    }
    if entity == state.controlled {
        interactions.push(InteractionType::Controlled);
        if inventory::is_holding::<&Medkit>(world, entity) {
            interactions.push(InteractionType::UseMedkit);
        }
    }
    if entity_ref.satisfies::<&ShipControls>() {
        interactions.push(InteractionType::ShipControls);
    }
    if entity_ref.satisfies::<&Shopkeeper>() {
        interactions.push(InteractionType::Shopkeeper);
    }
    if entity_ref.satisfies::<&Recruitable>() {
        interactions.push(InteractionType::Recruitable);
    }
    if entity_ref.satisfies::<&IsFoe>() {
        interactions.push(InteractionType::Foe);
    }
    interactions
}

pub fn for_priced_item(priced_item: &PricedItem, sellable_items: &[NameData]) -> Vec<Suggestion> {
    vec![
        simple!("buy {}", priced_item.item.noun_data().singular()),
        simple!("status"),
        simple!("exit"),
        recursive!(sellable_items.iter().map(NameData::base), "sell {}"),
        recursive!(
            sellable_items
                .iter()
                .filter(|name| sellable_items
                    .iter()
                    .filter(|other_name| other_name == name)
                    .count()
                    > 1)
                .map(NameData::plural),
            "sell all {}"
        ),
    ]
}

pub fn for_location_choice(choice: &Choice) -> Vec<Suggestion> {
    choice
        .alternatives()
        .into_iter()
        .map(Suggestion::Simple)
        .collect()
}

pub fn sorted_without_duplicates(
    suggestions: impl IntoIterator<Item = Suggestion>,
) -> Vec<Suggestion> {
    let mut suggestions = suggestions
        .into_iter()
        .filter(|suggestion| !suggestion.is_empty())
        .map(Suggestion::flatten)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    suggestions.sort();
    suggestions
}
