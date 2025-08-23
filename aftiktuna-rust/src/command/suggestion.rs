use crate::core::area::ShipControls;
use crate::core::inventory::Container;
use crate::core::item::{CanWield, ItemType};
use crate::core::name::{Name, NameData};
use crate::core::store::Shopkeeper;
use crate::core::{
    BlockType, Character, CrewMember, Door, FortunaChest, Hostile, Recruitable, Waiting, status,
};
use crate::game_loop::GameState;
use crate::location::Choice;
use crate::view::StoreStockView;
use crate::view::area::ItemProfile;
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
        Some(self.cmp(other))
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
    Container,
    Wieldable,
    Door,
    Forceable,
    ShipControls,
    Openable,
    CrewMember,
    Controlled,
    Shopkeeper,
    Recruitable,
    Talkable,
    Waiting,
    Following,
    Foe,
    Tameable,
}

impl InteractionType {
    pub fn commands(self, name: &str, inventory: &[ItemProfile]) -> Vec<Suggestion> {
        let name = name.to_lowercase();
        match self {
            InteractionType::Item => vec![simple!("take {name}"), simple!("check {name}")],
            InteractionType::Container => vec![simple!("search {name}")],
            InteractionType::Wieldable => vec![simple!("wield {name}")],
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
                    recursive!(
                        inventory
                            .iter()
                            .filter(|item| item.is_usable)
                            .map(ItemProfile::name),
                        "use {}"
                    ),
                    recursive!(inventory.iter().map(ItemProfile::name), "check {}"),
                ]
            }
            InteractionType::Shopkeeper => vec![simple!("trade")],
            InteractionType::Recruitable => {
                vec![simple!("recruit {name}")]
            }
            InteractionType::Talkable => {
                vec![simple!("talk to {name}")]
            }
            InteractionType::Waiting => vec![simple!("tell {name} to follow")],
            InteractionType::Following => vec![simple!("tell {name} to wait")],
            InteractionType::Foe => vec![simple!("attack {name}")],
            InteractionType::Tameable => vec![simple!("tame {name}")],
        }
    }
}

pub fn interactions_for(entity: Entity, state: &GameState) -> Vec<InteractionType> {
    let mut interactions = Vec::new();
    let world = &state.world;
    let entity_ref = world.entity(entity).unwrap();

    if entity_ref.satisfies::<&ItemType>() {
        interactions.push(InteractionType::Item);
    }
    if entity_ref.satisfies::<&CanWield>() {
        interactions.push(InteractionType::Wieldable);
    }

    if entity_ref.satisfies::<&Container>() {
        interactions.push(InteractionType::Container);
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
    if entity != state.controlled && entity_ref.satisfies::<(&CrewMember, &Character)>() {
        interactions.push(InteractionType::CrewMember);
        if entity_ref.satisfies::<&Waiting>() {
            interactions.push(InteractionType::Waiting);
        } else {
            interactions.push(InteractionType::Following);
        }
    }
    if entity == state.controlled {
        interactions.push(InteractionType::Controlled);
    }
    if entity_ref.satisfies::<&ShipControls>() {
        interactions.push(InteractionType::ShipControls);
    }
    if entity_ref.satisfies::<&Shopkeeper>() {
        interactions.push(InteractionType::Shopkeeper);
    }
    if entity_ref.satisfies::<(&Recruitable, &Character)>() {
        interactions.push(InteractionType::Recruitable);
    }
    if entity != state.controlled
        && entity_ref.satisfies::<(&Name, &Character)>()
        && status::is_alive_ref(entity_ref)
    {
        interactions.push(InteractionType::Talkable);
    }
    if entity_ref.has::<Hostile>() && status::is_alive_ref(entity_ref) {
        interactions.push(InteractionType::Foe);
        if entity_ref.has::<Recruitable>() {
            interactions.push(InteractionType::Tameable);
        }
    }
    interactions
}

pub fn for_store(
    clicked_stock: Option<&StoreStockView>,
    sellable_items: &[NameData],
) -> Vec<Suggestion> {
    let mut suggestions = vec![
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
    ];
    if let Some(clicked_stock) = clicked_stock
        && !clicked_stock.quantity.is_zero()
    {
        suggestions.push(simple!("buy {}", clicked_stock.item_noun.singular()));
    }
    suggestions
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
