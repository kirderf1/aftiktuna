use crate::action::trade::Points;
use crate::action::{item, CrewMember};
use crate::status::{Health, Stats};
use crate::view::{capitalize, DisplayInfo};
use hecs::{Entity, World};

pub struct Cache {
    points: i32,
    character_cache: CharacterCache,
}
struct CharacterCache {
    character_id: Entity,
    health: f32,
    wielded: Option<Entity>,
    inventory: Vec<Entity>,
}

pub fn print_full_status(world: &World, character: Entity) {
    print_points(world, character, None);

    println!("Crew:");
    for (character, _) in world.query::<()>().with::<&CrewMember>().iter() {
        println!(
            "{} (Aftik):",
            capitalize(DisplayInfo::find_definite_name(world, character).as_str())
        );
        print_stats(world, character);
        print_character_without_cache(world, character);
    }
}

pub fn print_with_cache(world: &World, character: Entity, cache: &mut Cache) {
    cache.points = print_points(world, character, Some(cache.points));
    print_character_with_cache(world, character, &mut cache.character_cache);
}

pub fn print_without_cache(world: &World, character: Entity) -> Cache {
    let points = print_points(world, character, None);
    let character_cache = print_character_without_cache(world, character);
    Cache {
        points,
        character_cache,
    }
}

fn print_points(world: &World, character: Entity, prev_points: Option<i32>) -> i32 {
    let crew = world.get::<&CrewMember>(character).unwrap().0;
    let points = world.get::<&Points>(crew).unwrap().0;

    if Some(points) == prev_points {
        return points;
    }

    println!("Crew points: {}p", points);

    points
}

fn print_character_with_cache(world: &World, character: Entity, cache: &mut CharacterCache) {
    if cache.character_id == character {
        cache.health = print_health(world, character, Some(cache.health));
        cache.wielded = print_wielded(world, character, Some(cache.wielded));
        cache.inventory = print_inventory(world, character, Some(&cache.inventory));
    } else {
        *cache = print_character_without_cache(world, character);
    }
}

fn print_character_without_cache(world: &World, character: Entity) -> CharacterCache {
    let health = print_health(world, character, None);
    let wielded = print_wielded(world, character, None);
    let inventory = print_inventory(world, character, None);
    CharacterCache {
        character_id: character,
        health,
        wielded,
        inventory,
    }
}

fn print_stats(world: &World, character: Entity) {
    let stats = world.get::<&Stats>(character).unwrap();
    println!(
        "Strength: {}   Endurance: {}   Agility: {}",
        stats.strength, stats.endurance, stats.agility
    );
}

const BAR_LENGTH: u16 = 10;

fn print_health(world: &World, character: Entity, prev_health: Option<f32>) -> f32 {
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
    println!("Health: {}", bar);
    health
}

fn print_wielded(
    world: &World,
    character: Entity,
    prev_wielded: Option<Option<Entity>>,
) -> Option<Entity> {
    let wielded = item::get_wielded(world, character);

    if Some(wielded) == prev_wielded {
        return wielded;
    }

    match wielded {
        None => println!("Wielding: Nothing"),
        Some(item) => println!(
            "Wielding: {}",
            capitalize(&DisplayInfo::find_name(world, item))
        ),
    }
    wielded
}

fn print_inventory(
    world: &World,
    character: Entity,
    prev_inv: Option<&Vec<Entity>>,
) -> Vec<Entity> {
    let mut inventory = item::get_inventory(world, character);
    inventory.sort();

    if Some(&inventory) == prev_inv {
        return inventory;
    }

    if inventory.is_empty() {
        println!("Inventory: Empty");
    } else {
        println!(
            "Inventory: {}",
            inventory
                .iter()
                .map(|item| capitalize(&DisplayInfo::find_name(world, *item)))
                .collect::<Vec<String>>()
                .join(", ")
        );
    }
    inventory
}
