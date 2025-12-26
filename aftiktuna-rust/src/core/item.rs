use super::display::ModelId;
use crate::asset::GameAssets;
use crate::core::name::NounId;
use crate::view::text;
use hecs::{Component, Entity, EntityBuilder, EntityRef, World};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Tool {
    Crowbar,
    Blowtorch,
}

impl Tool {
    pub fn into_message(self, character_name: &str) -> String {
        match self {
            Tool::Crowbar => format!(
                "{} used their crowbar and forced open the door.",
                character_name
            ),
            Tool::Blowtorch => format!(
                "{} used their blowtorch and cut open the door.",
                character_name
            ),
        }
    }

    pub fn matches(self, item_type: &ItemTypeId) -> bool {
        match self {
            Tool::Crowbar => item_type.is_crowbar(),
            Tool::Blowtorch => item_type.is_blowtorch(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Price(i32);

impl Price {
    pub fn buy_price(&self) -> i32 {
        self.0
    }

    pub fn sell_price(&self) -> i32 {
        self.0 - self.0 / 4
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemTypeId(String);

impl ItemTypeId {
    pub fn is_fuel_can(&self) -> bool {
        self.0 == "fuel_can"
    }
    const FOOD_RATION: &'static str = "food_ration";
    pub fn food_ration() -> Self {
        Self(Self::FOOD_RATION.into())
    }
    pub fn is_food_ration(&self) -> bool {
        self.0 == Self::FOOD_RATION
    }
    const CROWBAR: &'static str = "crowbar";
    pub fn crowbar() -> Self {
        Self(Self::CROWBAR.into())
    }
    pub fn is_crowbar(&self) -> bool {
        self.0 == Self::CROWBAR
    }
    pub fn is_blowtorch(&self) -> bool {
        self.0 == "blowtorch"
    }
    pub fn is_medkit(&self) -> bool {
        self.0 == "medkit"
    }
    pub fn is_black_orb(&self) -> bool {
        self.0 == "black_orb"
    }
    pub fn is_four_leaf_clover(&self) -> bool {
        self.0 == "four_leaf_clover"
    }

    pub fn noun_id(&self) -> NounId {
        NounId(self.0.clone())
    }

    pub fn model_id(&self) -> ModelId {
        ModelId::item(&self.0)
    }

    pub(crate) fn spawn(&self, world: &mut World, location: impl Component) -> Entity {
        let mut builder = EntityBuilder::new();
        builder
            .add::<ItemTypeId>(self.clone())
            .add::<ModelId>(self.model_id())
            .add::<NounId>(self.noun_id())
            .add(location);

        world.spawn(builder.build())
    }
}

impl Display for ItemTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub(crate) fn description(item_ref: EntityRef, assets: &GameAssets) -> Vec<String> {
    let mut messages = Vec::new();
    messages.push(format!(
        "{}:",
        text::capitalize(
            assets
                .noun_data_map
                .lookup(&item_ref.get::<&NounId>().unwrap())
                .singular()
        )
    ));

    let item_type = item_ref.get::<&ItemTypeId>().unwrap();
    let item_type_data = assets.item_type_map.get(&item_type);

    if let Some(weapon_properties) = item_type_data.and_then(|data| data.weapon) {
        messages.push(format!("Weapon value: {}", weapon_properties.damage_mod));
    }

    if let Some(extra_description) = item_type_data.and_then(|data| data.extra_description.as_ref())
    {
        messages.push(extra_description.into());
    }

    if item_type_data.is_some_and(|data| data.price.is_some()) {
        messages.push("Can be sold at a store.".into());
    }
    messages
}
