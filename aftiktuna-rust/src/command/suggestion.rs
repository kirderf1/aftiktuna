use crate::action::combat::IsFoe;
use crate::action::door::{BlockType, Door};
use crate::action::trade::{PricedItem, Shopkeeper};
use crate::action::{CrewMember, FortunaChest, Recruitable, Waiting};
use crate::core::area::ShipControls;
use crate::core::item::{CanWield, Item, Medkit};
use crate::core::{inventory, GameState};
use hecs::Entity;
use serde::{Deserialize, Serialize};

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
    pub fn commands(self, name: &str) -> Vec<String> {
        let name = name.to_lowercase();
        match self {
            InteractionType::Item => vec![format!("take {name}")],
            InteractionType::Wieldable => vec![format!("wield {name}")],
            InteractionType::UseMedkit => vec!["use medkit".to_owned()],
            InteractionType::Door => vec![format!("enter {name}")],
            InteractionType::Forceable => vec![format!("force {name}")],
            InteractionType::ShipControls => {
                vec!["launch ship".to_owned(), "refuel ship".to_owned()]
            }
            InteractionType::Openable => vec![format!("open {name}")],
            InteractionType::CrewMember => vec![
                format!("control {name}"),
                "status".to_owned(),
                "rest".to_owned(),
                format!("talk to {name}"),
            ],
            InteractionType::Controlled => {
                vec!["status".to_owned(), "rest".to_owned(), "wait".to_owned()]
            }
            InteractionType::Shopkeeper => vec!["trade".to_owned()],
            InteractionType::Recruitable => {
                vec![format!("recruit {name}"), format!("talk to {name}")]
            }
            InteractionType::Waiting => vec![format!("tell {name} to follow")],
            InteractionType::Following => vec![format!("tell {name} to wait")],
            InteractionType::Foe => vec![format!("attack {name}")],
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

pub fn for_priced_item(priced_item: &PricedItem) -> Vec<String> {
    vec![
        format!("buy {}", priced_item.item.noun_data().singular()),
        "status".to_owned(),
        "exit".to_owned(),
    ]
}
