use super::LocationGenContext;
use crate::asset::location::creature::{
    AftikCorpseData, AttributeChoice, CharacterInteraction, CreatureSpawnData, NpcSpawnData,
    ShopkeeperSpawnData, StockDefinition,
};
use crate::asset::{self, AftikProfile};
use crate::core::behavior::{Character, EncounterDialogue, Hostile, Recruitable, Wandering};
use crate::core::display::OrderWeight;
use crate::core::name::{Name, NounId};
use crate::core::position::{Direction, Large, OccupiesSpace, Pos};
use crate::core::status::{CreatureAttribute, Health, Stamina};
use crate::core::store::{Shopkeeper, StockQuantity, StoreStock};
use crate::core::{Species, inventory};
use hecs::{EntityBuilder, World};
use rand::Rng;
use rand::seq::IndexedRandom;
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
    let health = Health::from_fraction(spawn_data.health);
    let attribute = evaluate_attribute(spawn_data.attribute, &mut gen_context.rng);
    let is_alive = health.is_alive();
    let aggressive = spawn_data
        .aggressive
        .unwrap_or_else(|| spawn_data.creature.is_aggressive_by_default());
    let direction = spawn_data
        .direction
        .unwrap_or_else(|| Direction::towards_center(pos, &gen_context.world));
    let mut stats = spawn_data.creature.default_stats();

    let mut builder = species_builder_base(spawn_data.creature.species());
    if let Some(attribute) = attribute {
        attribute.adjust_stats(&mut stats);
        builder.add(attribute);
    }

    builder.add_bundle((pos, direction, health, Stamina::with_max(&stats), stats));

    if let Some(tag) = spawn_data.tag.clone() {
        builder.add(tag);
    }

    if is_alive {
        builder.add_bundle((OccupiesSpace, Hostile { aggressive }));
    }

    if spawn_data.wandering {
        builder.add(Wandering);
    }

    if spawn_data.creature.is_tameable() {
        builder.add(Recruitable);
    }

    gen_context.world.spawn(builder.build());
}

pub(super) fn place_npc(spawn_data: &NpcSpawnData, pos: Pos, gen_context: &mut LocationGenContext) {
    let Some(profile) = spawn_data
        .profile
        .clone()
        .unwrap(&mut gen_context.character_profiles, &mut gen_context.rng)
    else {
        return;
    };
    let direction = spawn_data
        .direction
        .unwrap_or_else(|| Direction::towards_center(pos, &gen_context.world));

    let mut builder = aftik_builder_with_stats(profile, false);
    builder.add_bundle((pos, direction));
    match &spawn_data.interaction {
        CharacterInteraction::Recruitable => {
            builder.add(Recruitable);
        }
        CharacterInteraction::GivesHuntReward(gives_hunt_reward) => {
            builder.add(gives_hunt_reward.clone());
        }
        CharacterInteraction::Hostile { encounter_dialogue } => {
            builder.add(Hostile { aggressive: true });
            if let Some(dialogue_node) = encounter_dialogue {
                builder.add(EncounterDialogue(dialogue_node.clone()));
            }
        }
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
        asset::remove_random_profile(&mut gen_context.character_profiles, &mut gen_context.rng)
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
) -> EntityBuilder {
    let mut builder = species_builder_base(Species::Aftik);
    builder.add_bundle((
        profile.color,
        Name {
            name: profile.name,
            is_known: is_name_known,
        },
        Health::from_fraction(1.),
        Stamina::with_max(&profile.stats),
        OccupiesSpace,
        profile.stats,
        profile.traits,
        Character,
    ));
    builder
}

fn species_builder_base(species: Species) -> EntityBuilder {
    let mut builder = EntityBuilder::new();
    builder.add_bundle((
        species,
        species.model_id(),
        species.noun_id(),
        OrderWeight::Creature,
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

    world.spawn((
        Species::Aftik.model_id(),
        OrderWeight::Creature,
        spawn_data.color.clone(),
        NounId::from("shopkeeper"),
        pos,
        direction,
        Shopkeeper(stock),
    ));
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
