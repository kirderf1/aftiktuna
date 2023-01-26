use crate::action::combat::IsFoe;
use crate::action::trade::{PricedItem, Shopkeeper};
use crate::action::{CrewMember, Recruitable};
use crate::item;
use crate::position::{MovementBlocking, Pos};
use crate::status::{Health, Stamina, Stats};
use crate::view::{DisplayInfo, NameData};
use hecs::{Entity, EntityBuilder, World};

pub fn spawn_crew_member(world: &mut World, crew: Entity, name: &str, stats: Stats) -> Entity {
    world.spawn(
        aftik_builder(name_display_info(name), stats)
            .add(CrewMember(crew))
            .build(),
    )
}

pub fn place_recruitable(world: &mut World, pos: Pos, name: &str, stats: Stats) {
    world.spawn(
        aftik_builder(
            DisplayInfo::new('A', NameData::from_noun("aftik"), 10),
            stats,
        )
        .add(Recruitable(name_display_info(name)))
        .add(pos)
        .build(),
    );
}

fn aftik_builder(display_info: DisplayInfo, stats: Stats) -> EntityBuilder {
    let mut builder = EntityBuilder::new();
    builder.add_bundle((
        display_info,
        Health::with_max(&stats),
        Stamina::with_max(&stats),
        stats,
    ));
    builder
}

fn name_display_info(name: &str) -> DisplayInfo {
    DisplayInfo::new(name.chars().next().unwrap(), NameData::from_name(name), 10)
}

pub fn place_goblin(world: &mut World, pos: Pos) {
    let stats = Stats::new(2, 4, 10);
    world.spawn((
        DisplayInfo::new('G', NameData::from_noun("goblin"), 10),
        pos,
        MovementBlocking,
        IsFoe,
        Health::with_max(&stats),
        Stamina::with_max(&stats),
        stats,
    ));
}

pub fn place_eyesaur(world: &mut World, pos: Pos) {
    let stats = Stats::new(7, 7, 4);
    world.spawn((
        DisplayInfo::new('E', NameData::from_noun("eyesaur"), 10),
        pos,
        MovementBlocking,
        IsFoe,
        Health::with_max(&stats),
        Stamina::with_max(&stats),
        stats,
    ));
}

pub fn place_azureclops(world: &mut World, pos: Pos) {
    let stats = Stats::new(15, 10, 4);
    world.spawn((
        DisplayInfo::new('Z', NameData::from_noun("azureclops"), 10),
        pos,
        MovementBlocking,
        IsFoe,
        Health::with_max(&stats),
        Stamina::with_max(&stats),
        stats,
    ));
}

pub fn place_shopkeeper(
    world: &mut World,
    pos: Pos,
    shop_items: &[item::Type],
) -> Result<(), String> {
    let stock = shop_items
        .iter()
        .map(|item| to_priced_item(*item))
        .collect::<Result<Vec<_>, String>>()?;
    world.spawn((
        DisplayInfo::new('S', NameData::from_noun("shopkeeper"), 15),
        pos,
        Shopkeeper(stock),
    ));
    Ok(())
}

fn to_priced_item(item: item::Type) -> Result<PricedItem, String> {
    item.price()
        .map(|price| PricedItem { item, price })
        .ok_or_else(|| {
            format!(
                "Cannot get a price from item \"{}\" to put in store",
                &item.name_data().base()
            )
        })
}
