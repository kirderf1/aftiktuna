use super::LocationGenContext;
use crate::asset::{self, AftikProfile, ProfileOrRandom};
use crate::core::display::{AftikColorId, ModelId, OrderWeight, Symbol};
use crate::core::name::{Name, Noun};
use crate::core::position::{Direction, OccupiesSpace, Pos};
use crate::core::status::{Health, Stamina, Stats};
use crate::core::store::{Shopkeeper, StockQuantity, StoreStock};
use crate::core::{item, CreatureAttribute, GivesHuntReward, Hostile, Recruitable, Tag};
use hecs::{EntityBuilder, World};
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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
    pub creature: Type,
    #[serde(default = "full_health")]
    pub health: f32,
    #[serde(default)]
    pub attribute: AttributeChoice,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aggressive: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<Tag>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<Direction>,
}

fn full_health() -> f32 {
    1.
}

impl CreatureSpawnData {
    pub(super) fn place(&self, pos: Pos, symbol: Symbol, gen_context: &mut LocationGenContext) {
        let health = Health::from_fraction(self.health);
        let attribute = self.attribute.evaluate(&mut gen_context.rng);
        let is_alive = health.is_alive();
        let aggressive = self
            .aggressive
            .unwrap_or_else(|| self.creature.is_aggressive_by_default());
        let direction = self
            .direction
            .unwrap_or_else(|| Direction::towards_center(pos, &gen_context.world));
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

        gen_context.world.spawn(builder.build());
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
    pub profile: ProfileOrRandom,
    pub interaction: CharacterInteraction,
    #[serde(default)]
    pub direction: Option<Direction>,
}

impl NpcSpawnData {
    pub(super) fn place(&self, pos: Pos, gen_context: &mut LocationGenContext) {
        let Some(profile) = self
            .profile
            .clone()
            .unwrap(&mut gen_context.character_profiles, &mut gen_context.rng)
        else {
            return;
        };
        let direction = self
            .direction
            .unwrap_or_else(|| Direction::towards_center(pos, &gen_context.world));

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
        gen_context.world.spawn(builder.build());
    }
}

#[derive(Serialize, Deserialize)]
pub struct AftikCorpseData {
    #[serde(default)]
    pub color: Option<AftikColorId>,
    #[serde(default)]
    pub direction: Option<Direction>,
}

impl AftikCorpseData {
    pub(super) fn place(&self, pos: Pos, gen_context: &mut LocationGenContext) {
        let Some(color) = self.color.clone().or_else(|| {
            asset::remove_random_profile(&mut gen_context.character_profiles, &mut gen_context.rng)
                .map(|profile| profile.color)
        }) else {
            return;
        };
        let direction = self
            .direction
            .unwrap_or_else(|| Direction::towards_center(pos, &gen_context.world));

        gen_context.world.spawn(
            aftik_builder(color)
                .add_bundle((Health::from_fraction(0.), pos, direction))
                .build(),
        );
    }
}

pub(crate) fn aftik_builder_with_stats(
    profile: AftikProfile,
    is_name_known: bool,
) -> EntityBuilder {
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
    pub stock: Vec<StockDefinition>,
    pub color: AftikColorId,
    #[serde(default)]
    pub direction: Option<Direction>,
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
    pub item: item::Type,
    #[serde(default)]
    pub price: Option<item::Price>,
    #[serde(default)]
    pub quantity: Option<StockQuantity>,
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

pub(super) fn align_aggressiveness(world: &mut World) {
    let areas_with_aggressive_creatures = world
        .query::<(&Pos, &Hostile)>()
        .iter()
        .filter(|&(_, (_, hostile))| hostile.aggressive)
        .map(|(_, (pos, _))| pos.get_area())
        .collect::<HashSet<_>>();
    for (_, (pos, hostile)) in world.query_mut::<(&Pos, &mut Hostile)>().into_iter() {
        hostile.aggressive |= areas_with_aggressive_creatures.contains(&pos.get_area());
    }
}
