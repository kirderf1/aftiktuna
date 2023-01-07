use crate::action::{item, CrewMember};
use crate::status::{Health, Stats};
use crate::view::{capitalize, DisplayInfo};
use hecs::{Entity, World};

pub struct Cache {
    character_id: Entity,
    health: f32,
    wielded: Option<Entity>,
    inventory: Vec<Entity>,
}

pub fn print_full_status(world: &World) {
    println!("Crew:");
    for (aftik, _) in world.query::<()>().with::<&CrewMember>().iter() {
        println!(
            "{} (Aftik):",
            capitalize(DisplayInfo::find_definite_name(world, aftik).as_str())
        );
        print_stats(world, aftik);
        print_without_cache(world, aftik);
    }
}

pub fn print_with_cache(world: &World, aftik: Entity, cache: &mut Cache) {
    if cache.character_id == aftik {
        cache.health = print_health(world, aftik, Some(cache.health));
        cache.wielded = print_wielded(world, aftik, Some(cache.wielded));
        cache.inventory = print_inventory(world, aftik, Some(&cache.inventory));
    } else {
        *cache = print_without_cache(world, aftik);
    }
}

pub fn print_without_cache(world: &World, aftik: Entity) -> Cache {
    let health = print_health(world, aftik, None);
    let wielded = print_wielded(world, aftik, None);
    let inventory = print_inventory(world, aftik, None);
    Cache {
        character_id: aftik,
        health,
        wielded,
        inventory,
    }
}

fn print_stats(world: &World, aftik: Entity) {
    let stats = world.get::<&Stats>(aftik).unwrap();
    println!(
        "Strength: {}   Endurance: {}   Agility: {}",
        stats.strength, stats.endurance, stats.agility
    );
}

const BAR_LENGTH: u16 = 10;

fn print_health(world: &World, aftik: Entity, prev_health: Option<f32>) -> f32 {
    let health = world.get::<&Health>(aftik).unwrap().as_fraction();

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
    aftik: Entity,
    prev_wielded: Option<Option<Entity>>,
) -> Option<Entity> {
    let wielded = item::get_wielded(world, aftik);

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

fn print_inventory(world: &World, aftik: Entity, prev_inv: Option<&Vec<Entity>>) -> Vec<Entity> {
    let mut inventory = item::get_inventory(world, aftik);
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
