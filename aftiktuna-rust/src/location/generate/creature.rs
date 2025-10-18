use super::LocationGenContext;
use crate::asset::location::creature::{
    AftikCorpseData, AttributeChoice, CharacterInteraction, CreatureSpawnData, NpcSpawnData,
    ShopkeeperSpawnData, StockDefinition,
};
use crate::asset::{self, AftikProfile};
use crate::core::behavior::{
    Character, EncounterDialogue, GivesHuntReward, Hostile, Recruitable, Talk, TalkState,
};
use crate::core::display::AftikColorId;
use crate::core::name::{Name, NounId};
use crate::core::position::{Direction, Large, OccupiesSpace, Pos};
use crate::core::status::{
    ChangedStats, CreatureAttribute, Health, Morale, Stamina, StatChanges, Stats, Trait, Traits,
};
use crate::core::store::{Shopkeeper, StockQuantity, StoreStock};
use crate::core::{Species, inventory};
use hecs::{EntityBuilder, World};
use rand::Rng;
use rand::seq::{IndexedRandom, SliceRandom};
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
) {
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
    let health = Health::from_fraction(*health);
    let attribute = evaluate_attribute(*attribute, &mut gen_context.rng);
    let is_alive = health.is_alive();
    let aggressive = aggressive.unwrap_or_else(|| creature.is_aggressive_by_default());
    let direction = direction.unwrap_or_else(|| Direction::towards_center(pos, &gen_context.world));
    let mut stats = stats.clone().unwrap_or(creature.species().default_stats());

    let mut builder = species_builder_base(creature.species());
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

    if creature.is_tameable() {
        builder.add(Recruitable);
    }

    gen_context.world.spawn(builder.build());
}

pub(super) fn place_npc(spawn_data: &NpcSpawnData, pos: Pos, gen_context: &mut LocationGenContext) {
    let Some(profile) = spawn_data.profile.clone().unwrap(
        &mut gen_context.character_profiles,
        &mut gen_context.rng,
        &used_aftik_colors(&mut gen_context.world),
    ) else {
        return;
    };
    let direction = spawn_data
        .direction
        .unwrap_or_else(|| Direction::towards_center(pos, &gen_context.world));

    let mut builder = aftik_builder_with_stats(profile, false, &mut gen_context.rng);
    builder.add_bundle((pos, direction));
    if let Some(tag) = spawn_data.tag.clone() {
        builder.add(tag);
    }
    match &spawn_data.interaction {
        CharacterInteraction::Recruitable => {
            builder.add(Recruitable);
        }
        CharacterInteraction::Talk(dialogue_node) => {
            builder.add(Talk(dialogue_node.clone()));
        }
        CharacterInteraction::GivesHuntReward(gives_hunt_reward) => {
            builder.add(GivesHuntReward::clone(gives_hunt_reward));
        }
        CharacterInteraction::Hostile { encounter_dialogue } => {
            builder.add(Hostile { aggressive: true });
            if let Some(dialogue_node) = encounter_dialogue {
                builder.add(EncounterDialogue(dialogue_node.clone()));
            }
        }
    }
    if let Some(background_dialogue) = spawn_data.background_dialogue.clone() {
        builder.add(background_dialogue);
    }

    let npc = gen_context.world.spawn(builder.build());
    if let Some(item_type) = spawn_data.wielded_item {
        item_type.spawn(&mut gen_context.world, inventory::Held::in_hand(npc));
    }
}

pub(super) fn place_corpse(
    spawn_data: &AftikCorpseData,
    pos: Pos,
    gen_context: &mut LocationGenContext,
) {
    let Some(color) = spawn_data.color.clone().or_else(|| {
        asset::remove_random_profile(
            &mut gen_context.character_profiles,
            &mut gen_context.rng,
            &used_aftik_colors(&mut gen_context.world),
        )
        .map(|profile| profile.color)
    }) else {
        return;
    };
    let direction = spawn_data
        .direction
        .unwrap_or_else(|| Direction::towards_center(pos, &gen_context.world));

    gen_context.world.spawn(
        species_builder_base(Species::Aftik)
            .add_bundle((color, Health::from_fraction(0.), pos, direction))
            .build(),
    );
}

pub(crate) fn aftik_builder_with_stats(
    profile: AftikProfile,
    is_name_known: bool,
    rng: &mut impl Rng,
) -> EntityBuilder {
    let AftikProfile {
        name,
        color,
        stats,
        traits,
    } = profile;
    let traits = traits.unwrap_or_else(|| random_traits(rng));
    let stats = stats
        .unwrap_or_else(|| random_stats_from_base(Species::Aftik.default_stats(), &traits, rng));
    let mut builder = species_builder_base(Species::Aftik);
    builder
        .add::<AftikColorId>(color)
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
    builder
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
    let change_from_traits = traits
        .sorted_iter()
        .map(Trait::effect_on_generated_stats)
        .sum();
    if change_from_traits > 0 {
        for _ in 1..=change_from_traits {
            adjust_random_stat(&mut stats, 1, rng);
        }
    } else if change_from_traits < 0 {
        for _ in 1..=(-change_from_traits) {
            adjust_random_stat(&mut stats, -1, rng);
        }
    }

    for _ in 1..=8 {
        adjust_random_stat(&mut stats, -1, rng);
        adjust_random_stat(&mut stats, 1, rng);
    }

    stats
}

fn adjust_random_stat(stats: &mut Stats, amount: i16, rng: &mut impl Rng) {
    let mut stat_changes = [
        StatChanges {
            strength: amount,
            endurance: 0,
            agility: 0,
            luck: 0,
        },
        StatChanges {
            strength: 0,
            endurance: amount,
            agility: 0,
            luck: 0,
        },
        StatChanges {
            strength: 0,
            endurance: 0,
            agility: amount,
            luck: 0,
        },
        StatChanges {
            strength: 0,
            endurance: 0,
            agility: 0,
            luck: amount,
        },
    ];
    stat_changes.shuffle(rng);
    for attempted_change in stat_changes {
        if let Ok(ChangedStats) = stats.try_change_in_bounds(attempted_change) {
            return;
        }
    }
}

fn species_builder_base(species: Species) -> EntityBuilder {
    let mut builder = EntityBuilder::new();
    builder.add_bundle((
        species,
        species.model_id(),
        species.noun_id(),
        Direction::default(),
    ));
    if species.is_large() {
        builder.add(Large);
    }
    builder
}

pub(super) fn place_shopkeeper(
    spawn_data: &ShopkeeperSpawnData,
    pos: Pos,
    world: &mut World,
) -> Result<(), String> {
    let direction = spawn_data
        .direction
        .unwrap_or_else(|| Direction::towards_center(pos, world));
    let stock = spawn_data
        .stock
        .iter()
        .map(build_stock)
        .collect::<Result<Vec<_>, String>>()?;

    world.spawn(
        species_builder_base(Species::Aftik)
            .add_bundle((
                spawn_data.color.clone(),
                NounId::from("shopkeeper"),
                pos,
                direction,
                Shopkeeper(stock),
            ))
            .build(),
    );
    Ok(())
}

fn build_stock(
    &StockDefinition {
        item,
        price,
        quantity,
    }: &StockDefinition,
) -> Result<StoreStock, String> {
    let price = price.or_else(|| item.price()).ok_or_else(|| {
        format!(
            "Cannot get a price from item \"{}\" to put in store",
            item.noun_id().0
        )
    })?;
    let quantity = quantity.unwrap_or(StockQuantity::Unlimited);
    Ok(StoreStock {
        item,
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

pub(crate) fn used_aftik_colors(world: &mut World) -> Vec<&AftikColorId> {
    world
        .query_mut::<&AftikColorId>()
        .into_iter()
        .map(|(_, color)| color)
        .collect()
}
