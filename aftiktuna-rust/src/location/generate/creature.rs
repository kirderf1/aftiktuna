use crate::core::display::{AftikColorId, ModelId, OrderWeight, Symbol};
use crate::core::name::{Name, Noun};
use crate::core::position::{Direction, OccupiesSpace, Pos};
use crate::core::status::{Health, Stamina, Stats, Traits};
use crate::core::store::{Shopkeeper, StockQuantity, StoreStock};
use crate::core::{item, CreatureAttribute, GivesHuntReward, Hostile, Recruitable, Tag};
use hecs::{EntityBuilder, World};
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributeChoice {
    None,
    #[default]
    Random,
    #[serde(untagged)]
    Attribute(CreatureAttribute),
}

impl AttributeChoice {
    fn evaluate(self, rng: &mut impl Rng) -> Option<CreatureAttribute> {
        match self {
            AttributeChoice::None => None,
            AttributeChoice::Attribute(attribute) => Some(attribute),
            AttributeChoice::Random => {
                if rng.gen_bool(0.5) {
                    None
                } else {
                    [
                        CreatureAttribute::Muscular,
                        CreatureAttribute::Bulky,
                        CreatureAttribute::Agile,
                    ]
                    .choose(rng)
                    .copied()
                }
            }
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    Goblin,
    Eyesaur,
    Azureclops,
    Scarvie,
    VoraciousFrog,
}

impl Type {
    fn is_aggressive_by_default(self) -> bool {
        match self {
            Type::Goblin | Type::Eyesaur | Type::Scarvie => false,
            Type::Azureclops | Type::VoraciousFrog => true,
        }
    }

    fn default_stats(self) -> Stats {
        match self {
            Type::Goblin => Stats::new(2, 4, 10, 2),
            Type::Eyesaur => Stats::new(7, 7, 4, 2),
            Type::Azureclops => Stats::new(15, 10, 4, 2),
            Type::Scarvie => Stats::new(3, 2, 8, 1),
            Type::VoraciousFrog => Stats::new(8, 8, 3, 3),
        }
    }

    fn model_id(self) -> ModelId {
        ModelId::creature(match self {
            Type::Goblin => "goblin",
            Type::Eyesaur => "eyesaur",
            Type::Azureclops => "azureclops",
            Type::Scarvie => "scarvie",
            Type::VoraciousFrog => "voracious_frog",
        })
    }

    fn noun(self) -> Noun {
        match self {
            Type::Goblin => Noun::new("goblin", "goblins"),
            Type::Eyesaur => Noun::new("eyesaur", "eyesaurs"),
            Type::Azureclops => Noun::new("azureclops", "azureclopses"),
            Type::Scarvie => Noun::new("scarvie", "scarvies"),
            Type::VoraciousFrog => Noun::new("voracious frog", "voracious frogs"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CreatureSpawnData {
    creature: Type,
    #[serde(default = "full_health")]
    health: f32,
    #[serde(default)]
    attribute: AttributeChoice,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    aggressive: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tag: Option<Tag>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    direction: Option<Direction>,
}

fn full_health() -> f32 {
    1.
}

impl CreatureSpawnData {
    pub fn place(&self, pos: Pos, symbol: Symbol, world: &mut World, rng: &mut impl Rng) {
        let health = Health::from_fraction(self.health);
        let attribute = self.attribute.evaluate(rng);
        let is_alive = health.is_alive();
        let aggressive = self
            .aggressive
            .unwrap_or_else(|| self.creature.is_aggressive_by_default());
        let direction = self
            .direction
            .unwrap_or_else(|| Direction::towards_center(pos, world));
        let mut stats = self.creature.default_stats();

        let mut builder = EntityBuilder::new();
        if let Some(attribute) = attribute {
            attribute.adjust_stats(&mut stats);
            builder.add(attribute);
        }

        builder.add_bundle((
            self.creature.model_id(),
            self.creature.noun(),
            symbol,
            OrderWeight::Creature,
            pos,
            direction,
            health,
            Stamina::with_max(&stats),
            stats,
        ));

        if let Some(tag) = self.tag.clone() {
            builder.add(tag);
        }

        if is_alive {
            builder.add_bundle((OccupiesSpace, Hostile { aggressive }));
        }

        world.spawn(builder.build());
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProfileOrRandom {
    #[default]
    Random,
    #[serde(untagged)]
    Profile(AftikProfile),
}

impl ProfileOrRandom {
    pub fn unwrap(
        self,
        character_profiles: &mut Vec<AftikProfile>,
        rng: &mut impl Rng,
    ) -> Option<AftikProfile> {
        match self {
            ProfileOrRandom::Random => remove_random_profile(character_profiles, rng),
            ProfileOrRandom::Profile(profile) => Some(profile),
        }
    }
}

pub fn remove_random_profile(
    character_profiles: &mut Vec<AftikProfile>,
    rng: &mut impl Rng,
) -> Option<AftikProfile> {
    if character_profiles.is_empty() {
        eprintln!("Tried picking a random profile, but there were none left to choose.");
        return None;
    }
    let chosen_index = rng.gen_range(0..character_profiles.len());
    Some(character_profiles.swap_remove(chosen_index))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AftikProfile {
    name: String,
    color: AftikColorId,
    stats: Stats,
    #[serde(default)]
    traits: Traits,
}

impl From<AftikProfile> for AftikColorId {
    fn from(value: AftikProfile) -> Self {
        value.color
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CharacterInteraction {
    Recruitable,
    GivesHuntReward(GivesHuntReward),
}

#[derive(Serialize, Deserialize)]
pub struct NpcSpawnData {
    #[serde(default)]
    profile: ProfileOrRandom,
    interaction: CharacterInteraction,
    #[serde(default)]
    direction: Option<Direction>,
}

impl NpcSpawnData {
    pub fn place(
        &self,
        pos: Pos,
        world: &mut World,
        character_profiles: &mut Vec<AftikProfile>,
        rng: &mut impl Rng,
    ) {
        let Some(profile) = self.profile.clone().unwrap(character_profiles, rng) else {
            return;
        };
        let direction = self
            .direction
            .unwrap_or_else(|| Direction::towards_center(pos, world));

        let mut builder = aftik_builder_with_stats(profile, false);
        builder.add_bundle((pos, direction));
        match &self.interaction {
            CharacterInteraction::Recruitable => {
                builder.add(Recruitable);
            }
            CharacterInteraction::GivesHuntReward(gives_hunt_reward) => {
                builder.add(gives_hunt_reward.clone());
            }
        }
        world.spawn(builder.build());
    }
}

#[derive(Serialize, Deserialize)]
pub struct AftikCorpseData {
    #[serde(default)]
    color: Option<AftikColorId>,
    #[serde(default)]
    direction: Option<Direction>,
}

impl AftikCorpseData {
    pub fn place(
        &self,
        pos: Pos,
        world: &mut World,
        character_profiles: &mut Vec<AftikProfile>,
        rng: &mut impl Rng,
    ) {
        let Some(color) = self
            .color
            .clone()
            .or_else(|| remove_random_profile(character_profiles, rng).map(AftikColorId::from))
        else {
            return;
        };
        let direction = self
            .direction
            .unwrap_or_else(|| Direction::towards_center(pos, world));

        world.spawn(
            aftik_builder(color)
                .add_bundle((Health::from_fraction(0.), pos, direction))
                .build(),
        );
    }
}

pub fn aftik_builder_with_stats(profile: AftikProfile, is_name_known: bool) -> EntityBuilder {
    let mut builder = aftik_builder(profile.color);
    builder.add_bundle((
        Name {
            name: profile.name,
            is_known: is_name_known,
        },
        Health::from_fraction(1.),
        Stamina::with_max(&profile.stats),
        OccupiesSpace,
        profile.stats,
        profile.traits,
    ));
    builder
}

fn aftik_builder(color: AftikColorId) -> EntityBuilder {
    let mut builder = EntityBuilder::new();
    builder.add_bundle((
        ModelId::aftik(),
        color,
        OrderWeight::Creature,
        Noun::new("aftik", "aftiks"),
    ));
    builder
}

#[derive(Serialize, Deserialize)]
pub struct ShopkeeperSpawnData {
    stock: Vec<StockDefinition>,
    color: AftikColorId,
    #[serde(default)]
    direction: Option<Direction>,
}

impl ShopkeeperSpawnData {
    pub fn place(&self, pos: Pos, world: &mut World) -> Result<(), String> {
        let direction = self
            .direction
            .unwrap_or_else(|| Direction::towards_center(pos, world));
        let stock = self
            .stock
            .iter()
            .map(StockDefinition::build)
            .collect::<Result<Vec<_>, String>>()?;

        world.spawn((
            ModelId::aftik(),
            OrderWeight::Creature,
            self.color.clone(),
            Noun::new("shopkeeper", "shopkeepers"),
            pos,
            direction,
            Shopkeeper(stock),
        ));
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StockDefinition {
    item: item::Type,
    #[serde(default)]
    price: Option<item::Price>,
    #[serde(default)]
    quantity: Option<StockQuantity>,
}

impl StockDefinition {
    fn build(&self) -> Result<StoreStock, String> {
        let Self {
            item,
            price,
            quantity,
        } = *self;
        let price = price.or_else(|| item.price()).ok_or_else(|| {
            format!(
                "Cannot get a price from item \"{}\" to put in store",
                item.noun_data().singular()
            )
        })?;
        let quantity = quantity.unwrap_or(StockQuantity::Unlimited);
        Ok(StoreStock {
            item,
            price,
            quantity,
        })
    }
}
