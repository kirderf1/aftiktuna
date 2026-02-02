use super::LocationGenContext;
use crate::asset::location::creature::{
    AttributeChoice, CharacterCorpseData, CharacterInteraction, CreatureSpawnData, NpcSpawnData,
    StockDefinition,
};
use crate::asset::profile::CharacterProfile;
use crate::asset::{GameAssets, SpeciesData, SpeciesDataMap, SpeciesKind};
use crate::core::behavior::{
    self, Character, EncounterDialogue, GivesHuntRewardData, Hostile, Recruitable, Talk, TalkState,
};
use crate::core::display::{CreatureVariantSet, SpeciesColorId};
use crate::core::name::Name;
use crate::core::position::{Direction, Large, OccupiesSpace, Pos};
use crate::core::status::{
    ChangedStats, CreatureAttribute, Health, Morale, Stamina, Stats, Trait, Traits,
};
use crate::core::store::{Shopkeeper, StockQuantity, StoreStock};
use crate::core::{SpeciesId, Tag, inventory};
use hecs::{EntityBuilder, World};
use rand::Rng;
use rand::seq::{IndexedRandom, IteratorRandom};
use std::collections::HashSet;

fn evaluate_attribute(choice: AttributeChoice, rng: &mut impl Rng) -> Option<CreatureAttribute> {
    match choice {
        AttributeChoice::None => None,
        AttributeChoice::Attribute(attribute) => Some(attribute),
        AttributeChoice::Random => {
            if rng.random_bool(0.5) {
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

pub(super) fn place_creature(
    spawn_data: &CreatureSpawnData,
    pos: Pos,
    gen_context: &mut LocationGenContext,
) -> Result<(), String> {
    let CreatureSpawnData {
        creature,
        name,
        health,
        stats,
        attribute,
        aggressive,
        wandering,
        tag,
        direction,
    } = spawn_data;
    let species_data = gen_context
        .assets
        .species_data_map
        .get(creature)
        .ok_or_else(|| format!("Missing data for species: {creature}"))?;
    let &SpeciesKind::Fauna {
        agressive_by_default,
        tameable,
    } = &species_data.kind
    else {
        return Err(format!(
            "Tried spawning fauna with non-fauna species: {creature}"
        ));
    };

    let health = Health::from_fraction(*health);
    let attribute = evaluate_attribute(*attribute, &mut gen_context.rng);
    let is_alive = health.is_alive();
    let aggressive = aggressive.unwrap_or(agressive_by_default);
    let direction = direction.unwrap_or_else(|| Direction::towards_center(pos, &gen_context.world));
    let mut stats = stats.unwrap_or(species_data.default_stats);

    let mut builder = species_builder_base(creature.clone(), species_data, &mut gen_context.rng);

    if let Some(color_id) = gen_context
        .assets
        .color_map
        .available_ids(creature)
        .choose_stable(&mut gen_context.rng)
    {
        builder.add::<SpeciesColorId>(color_id.clone());
    }

    if let Some(attribute) = attribute {
        attribute.adjust_stats(&mut stats);
        builder.add(attribute);
    }

    builder.add_bundle((pos, direction, health, Stamina::with_max(&stats), stats));

    if let Some(name) = name {
        builder.add(Name::known(name));
    }

    if let Some(tag) = tag.clone() {
        builder.add(tag);
    }

    if is_alive {
        builder.add_bundle((OccupiesSpace, Hostile { aggressive }));
    }

    if let Some(wandering) = wandering.clone() {
        builder.add(wandering);
    }

    if tameable {
        builder.add(Recruitable);
    }

    gen_context.world.spawn(builder.build());
    Ok(())
}

pub(super) fn place_npc(
    spawn_data: &NpcSpawnData,
    pos: Pos,
    gen_context: &mut LocationGenContext,
) -> Result<(), String> {
    let NpcSpawnData {
        profile,
        health,
        tag,
        background,
        interaction,
        background_dialogue,
        wielded_item,
        direction,
    } = spawn_data;
    let Some(profile) = profile.clone().unwrap(
        &mut gen_context.character_names,
        &mut gen_context.aftik_color_names,
        &gen_context.assets.color_map,
        &mut gen_context.rng,
        |species| used_species_colors(&mut gen_context.world, species),
    ) else {
        return Ok(());
    };
    let direction = direction.unwrap_or_else(|| Direction::towards_center(pos, &gen_context.world));

    let mut builder = character_builder_with_stats(
        profile,
        false,
        &gen_context.assets.species_data_map,
        &mut gen_context.rng,
    )?;
    builder
        .add::<Pos>(pos)
        .add::<Direction>(direction)
        .add(Health::from_fraction(*health));
    if let Some(tag) = tag.clone() {
        builder.add::<Tag>(tag);
    }
    if let Some(background) = background.clone() {
        builder.add::<behavior::BackgroundId>(background);
    }
    match interaction {
        CharacterInteraction::Recruitable => {
            builder.add(Recruitable);
        }
        CharacterInteraction::Talk { dialogue } => {
            builder.add(Talk(dialogue.clone()));
        }
        CharacterInteraction::GivesHuntReward(gives_hunt_reward) => {
            builder.add::<GivesHuntRewardData>(gives_hunt_reward.cloned_data());
        }
        CharacterInteraction::Shopkeeper { stock } => {
            let stock = stock
                .iter()
                .map(|stock| build_stock(stock, gen_context.assets))
                .collect::<Result<Vec<_>, String>>()?;
            builder.add(Shopkeeper(stock));
        }
        CharacterInteraction::Hostile { encounter_dialogue } => {
            builder.add(Hostile { aggressive: true });
            if let Some(dialogue_node) = encounter_dialogue {
                builder.add(EncounterDialogue(dialogue_node.clone()));
            }
        }
    }
    if let Some(background_dialogue) = background_dialogue.clone() {
        builder.add(background_dialogue);
    }

    let npc = gen_context.world.spawn(builder.build());
    if let Some(item_type) = wielded_item {
        item_type.spawn(&mut gen_context.world, inventory::Held::in_hand(npc));
    }

    Ok(())
}

pub(super) fn place_corpse(
    spawn_data: &CharacterCorpseData,
    pos: Pos,
    gen_context: &mut LocationGenContext,
) -> Result<(), String> {
    let CharacterCorpseData {
        species,
        color,
        direction,
    } = spawn_data;
    let species_data = gen_context
        .assets
        .species_data_map
        .get(species)
        .ok_or_else(|| format!("Missing data for species: {}", species))?;
    if !matches!(species_data.kind, SpeciesKind::CharacterSpecies) {
        return Err(format!(
            "Tried spawning character with non-character species: {species}"
        ));
    }

    let Some(color) = color.clone().or_else(|| {
        let used_colors = used_species_colors(&mut gen_context.world, species);
        gen_context
            .assets
            .color_map
            .available_ids(species)
            .filter(|color| !used_colors.contains(color))
            .choose_stable(&mut gen_context.rng)
            .cloned()
    }) else {
        return Ok(());
    };
    let direction = direction.unwrap_or_else(|| Direction::towards_center(pos, &gen_context.world));

    gen_context.world.spawn(
        species_builder_base(species.clone(), species_data, &mut gen_context.rng)
            .add_bundle((color, Health::from_fraction(0.), pos, direction))
            .build(),
    );
    Ok(())
}

pub(crate) fn character_builder_with_stats(
    profile: CharacterProfile,
    is_name_known: bool,
    species_map: &SpeciesDataMap,
    rng: &mut impl Rng,
) -> Result<EntityBuilder, String> {
    let CharacterProfile {
        name,
        species,
        color,
        stats,
        traits,
    } = profile;
    let species_data = species_map
        .get(&species)
        .ok_or_else(|| format!("Missing data for species: {species}"))?;
    let SpeciesKind::CharacterSpecies = &species_data.kind else {
        return Err(format!(
            "Tried spawning character with non-character species: {species}"
        ));
    };

    let traits = traits.unwrap_or_else(|| random_traits(rng));
    let stats =
        stats.unwrap_or_else(|| random_stats_from_base(species_data.default_stats, &traits, rng));
    let mut builder = species_builder_base(species, species_data, rng);
    builder
        .add::<SpeciesColorId>(color)
        .add_bundle((
            Name {
                name,
                is_known: is_name_known,
            },
            Health::from_fraction(1.),
            Stamina::with_max(&stats),
            Morale::default(),
            TalkState::default(),
            OccupiesSpace,
            Character,
        ))
        .add::<Stats>(stats)
        .add::<Traits>(traits);
    Ok(builder)
}

fn random_traits(rng: &mut impl Rng) -> Traits {
    let mut traits = Vec::new();

    if let &Some(positive_trait) = [None, Some(Trait::GoodDodger), Some(Trait::FastHealer)]
        .choose(rng)
        .unwrap()
    {
        traits.push(positive_trait);
    }

    if let &Some(negative_trait) = [None, Some(Trait::Fragile), Some(Trait::BigEater)]
        .choose(rng)
        .unwrap()
    {
        traits.push(negative_trait);
    }

    traits.into()
}

fn random_stats_from_base(mut stats: Stats, traits: &Traits, rng: &mut impl Rng) -> Stats {
    let change_from_traits: i16 = traits
        .sorted_iter()
        .map(Trait::effect_on_generated_stats)
        .sum();
    let stats_sum_target = Stats::CHARACTER_STATS_SUM_TARGET + change_from_traits;
    while stats.sum() < stats_sum_target {
        let Ok(ChangedStats) = stats.adjust_random_in_bounds(1, rng) else {
            break;
        };
    }
    while stats.sum() > stats_sum_target {
        let Ok(ChangedStats) = stats.adjust_random_in_bounds(-1, rng) else {
            break;
        };
    }

    for _ in 1..=8 {
        let Ok(ChangedStats) = stats.adjust_random_in_bounds(-1, rng) else {
            break;
        };
        stats.adjust_random_in_bounds(1, rng).unwrap();
    }

    stats
}

fn species_builder_base(
    species_id: SpeciesId,
    species_data: &SpeciesData,
    rng: &mut impl Rng,
) -> EntityBuilder {
    let mut builder = EntityBuilder::new();

    let model_id = species_id.model_id();
    let noun_id = species_id.noun_id();
    builder.add_bundle((
        species_id,
        model_id,
        noun_id,
        Direction::default(),
        CreatureVariantSet::random_for_species(species_data, rng),
    ));
    if species_data.is_large {
        builder.add(Large);
    }
    builder
}

fn build_stock(
    StockDefinition {
        item,
        price,
        quantity,
    }: &StockDefinition,
    assets: &GameAssets,
) -> Result<StoreStock, String> {
    let price = price
        .or_else(|| assets.item_type_map.get(item).and_then(|data| data.price))
        .ok_or_else(|| {
            format!(
                "Cannot get a price from item \"{}\" to put in store",
                item.noun_id().0
            )
        })?;
    let quantity = quantity.unwrap_or(StockQuantity::Unlimited);
    Ok(StoreStock {
        item: item.clone(),
        price,
        quantity,
    })
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

pub(crate) fn used_species_colors<'a>(
    world: &'a mut World,
    expected_species: &SpeciesId,
) -> Vec<&'a SpeciesColorId> {
    world
        .query_mut::<(&SpeciesColorId, &SpeciesId)>()
        .into_iter()
        .filter(|&(_, (_, checked_species))| checked_species == expected_species)
        .map(|(_, (color, _))| color)
        .collect()
}
