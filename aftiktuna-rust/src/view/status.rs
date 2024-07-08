use super::text::{self, Messages};
use crate::core::area::{FuelAmount, Ship, ShipStatus};
use crate::core::item::FoodRation;
use crate::core::name::{self, NameData};
use crate::core::position::Pos;
use crate::core::status::{Health, Stats, Trait, Traits};
use crate::core::store::Points;
use crate::core::{inventory, CrewMember};
use crate::game_loop::GameState;
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Cache {
    points: Option<i32>,
    character_cache: Option<CharacterCache>,
}

#[derive(Serialize, Deserialize)]
struct CharacterCache {
    character_id: Entity,
    health: f32,
    wielded: Option<Entity>,
    inventory: Vec<Entity>,
}

pub fn print_full_status(messages: &mut Messages, state: &GameState) {
    maybe_print_points(&state.world, state.controlled, messages, None);

    match state.world.get::<&Ship>(state.ship).map(|ship| ship.status) {
        Ok(ShipStatus::Refueled) => messages.add("The ship is refueled and ready to launch"),
        Ok(ShipStatus::NeedFuel(FuelAmount::OneCan)) => {
            messages.add("The ship needs one more fuel can before it can launch")
        }
        Ok(ShipStatus::NeedFuel(FuelAmount::TwoCans)) => {
            messages.add("The ship needs two more fuel cans before it can launch")
        }
        _ => {}
    }

    let ration_count = state
        .world
        .query::<&Pos>()
        .with::<&FoodRation>()
        .iter()
        .filter(|&(_, pos)| pos.is_in(state.ship))
        .count();
    messages.add(format!("Food rations at ship: {ration_count}"));

    messages.add("Crew:");
    for (character, _) in state.world.query::<()>().with::<&CrewMember>().iter() {
        messages.add("");
        messages.add(format!(
            "{} (Aftik):",
            NameData::find(&state.world, character).definite()
        ));
        messages.add(stats_message(
            &state.world.get::<&Stats>(character).unwrap(),
        ));
        if let Some(traits) = state
            .world
            .get::<&Traits>(character)
            .ok()
            .filter(|traits| traits.has_traits())
        {
            messages.add(traits_message(&traits));
        }
        print_character_without_cache(&state.world, character, messages);
    }
}

pub fn changes_messages(
    world: &World,
    character: Entity,
    messages: &mut Messages,
    cache: &mut Cache,
) {
    cache.points = Some(maybe_print_points(world, character, messages, cache.points));
    if let Some(character_cache) = &mut cache.character_cache {
        print_character_with_cache(world, character, messages, character_cache);
    } else {
        cache.character_cache = Some(print_character_without_cache(world, character, messages));
    }
}

pub fn fetch_points(world: &World, character: Entity, cache: &mut Cache) -> i32 {
    let crew = world.get::<&CrewMember>(character).unwrap().0;
    let points = world.get::<&Points>(crew).unwrap().0;

    cache.points = Some(points);

    points
}

fn maybe_print_points(
    world: &World,
    character: Entity,
    messages: &mut Messages,
    prev_points: Option<i32>,
) -> i32 {
    let crew = world.get::<&CrewMember>(character).unwrap().0;
    let points = world.get::<&Points>(crew).unwrap().0;

    if Some(points) == prev_points {
        return points;
    }

    messages.add(format!("Crew points: {points}p"));

    points
}

fn print_character_with_cache(
    world: &World,
    character: Entity,
    messages: &mut Messages,
    cache: &mut CharacterCache,
) {
    if cache.character_id == character {
        cache.health = print_health(world, character, messages, Some(cache.health));
        cache.wielded = print_wielded(world, character, messages, Some(cache.wielded));
        cache.inventory = print_inventory(world, character, messages, Some(&cache.inventory));
    } else {
        *cache = print_character_without_cache(world, character, messages);
    }
}

fn print_character_without_cache(
    world: &World,
    character: Entity,
    messages: &mut Messages,
) -> CharacterCache {
    let health = print_health(world, character, messages, None);
    let wielded = print_wielded(world, character, messages, None);
    let inventory = print_inventory(world, character, messages, None);
    CharacterCache {
        character_id: character,
        health,
        wielded,
        inventory,
    }
}

fn stats_message(stats: &Stats) -> String {
    let Stats {
        strength,
        endurance,
        agility,
        luck,
    } = *stats;
    format!("Strength: {strength}   Endurance: {endurance}   Agility: {agility}   Luck: {luck}")
}

fn traits_message(traits: &Traits) -> String {
    format!(
        "Traits: {}",
        traits
            .sorted_iter()
            .map(Trait::name)
            .collect::<Vec<_>>()
            .join(" ")
    )
}

const BAR_LENGTH: u16 = 10;

fn print_health(
    world: &World,
    character: Entity,
    messages: &mut Messages,
    prev_health: Option<f32>,
) -> f32 {
    let health = world.get::<&Health>(character).unwrap().as_fraction();

    if Some(health) == prev_health {
        return health;
    }

    let bar = (0..BAR_LENGTH)
        .map(|i| {
            if f32::from(i) < f32::from(BAR_LENGTH) * health {
                '#'
            } else {
                '.'
            }
        })
        .collect::<String>();
    messages.add(format!("Health: {bar}"));
    health
}

fn print_wielded(
    world: &World,
    character: Entity,
    messages: &mut Messages,
    prev_wielded: Option<Option<Entity>>,
) -> Option<Entity> {
    let wielded = inventory::get_wielded(world, character);

    if Some(wielded) == prev_wielded {
        return wielded;
    }

    let wield_text = wielded.map_or_else(
        || "Nothing".to_string(),
        |item| text::capitalize(NameData::find(world, item).base()),
    );
    messages.add(format!("Wielding {wield_text}"));
    wielded
}

fn print_inventory(
    world: &World,
    character: Entity,
    messages: &mut Messages,
    prev_inv: Option<&Vec<Entity>>,
) -> Vec<Entity> {
    let mut inventory = inventory::get_inventory(world, character);
    inventory.sort();

    if Some(&inventory) == prev_inv {
        return inventory;
    }

    let inventory_text = if inventory.is_empty() {
        "Empty".to_string()
    } else {
        name::names_with_counts(
            inventory.iter().map(|item| NameData::find(world, *item)),
            name::Article::One,
            name::CountFormat::Numeric,
        )
        .join(", ")
    };
    messages.add(format!("Inventory: {inventory_text}"));
    inventory
}
