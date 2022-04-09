use crate::action::combat::Stats;
use crate::item::InInventory;
use crate::view::{capitalize, StatusCache};
use crate::{view, DisplayInfo, Health};
use hecs::{Entity, With, World};

pub fn print_full_status(world: &World, aftik: Entity) {
    println!(
        "{} (Aftik):",
        capitalize(DisplayInfo::find_definite_name(world, aftik).as_str())
    );
    print_stats(world, aftik);
    print_status(world, aftik, &mut None);
}

pub fn print_status(world: &World, aftik: Entity, cache: &mut Option<StatusCache>) {
    if let Some(cache) = cache {
        cache.health = print_health(world, aftik, Some(cache.health));
        cache.inventory = print_inventory(world, aftik, Some(&cache.inventory));
    } else {
        let health = print_health(world, aftik, None);
        let inventory = print_inventory(world, aftik, None);
        *cache = Some(StatusCache { health, inventory });
    }
}

fn print_stats(world: &World, aftik: Entity) {
    let stats = world.get::<Stats>(aftik).unwrap();
    println!(
        "Strength: {}   Endurance: {}   Agility: {}",
        stats.strength, stats.endurance, stats.agility
    );
}

const BAR_LENGTH: u16 = 10;

fn print_health(world: &World, aftik: Entity, prev_health: Option<f32>) -> f32 {
    let health = world.get::<Health>(aftik).unwrap().as_fraction();

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

fn print_inventory(world: &World, _aftik: Entity, prev_inv: Option<&Vec<Entity>>) -> Vec<Entity> {
    let mut query = world.query::<With<InInventory, &DisplayInfo>>();

    let mut inventory = query.iter().map(|(entity, _)| entity).collect::<Vec<_>>();
    inventory.sort();

    if Some(&inventory) == prev_inv {
        return inventory;
    }

    if inventory.is_empty() {
        println!("Inventory: Empty");
    } else {
        println!(
            "Inventory: {}",
            query
                .iter()
                .map(|(_, info)| view::capitalize(info.name()))
                .collect::<Vec<String>>()
                .join(", ")
        );
    }
    inventory
}
