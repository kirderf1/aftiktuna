use crate::action::combat::IsFoe;
use crate::action::trade::{PricedItem, Shopkeeper};
use crate::action::{CrewMember, Recruitable};
use crate::core::item;
use crate::core::position::{Direction, MovementBlocking, Pos};
use crate::core::status::{Health, Stamina, Stats};
use crate::view;
use crate::view::{AftikColor, DisplayInfo, NameData, TextureType};
use hecs::{Entity, EntityBuilder, World};

pub fn spawn_crew_member(
    world: &mut World,
    crew: Entity,
    name: &str,
    stats: Stats,
    color: AftikColor,
) -> Entity {
    world.spawn(
        aftik_builder(
            view::name_display_info(TextureType::Aftik, name),
            NameData::from_name(name),
            stats,
        )
        .add(color)
        .add(CrewMember(crew))
        .build(),
    )
}

pub fn place_recruitable(world: &mut World, pos: Pos, name: &str, stats: Stats, color: AftikColor) {
    world.spawn(
        aftik_builder(
            DisplayInfo::new('A', TextureType::Aftik, 10),
            NameData::from_noun("aftik", "aftiks"),
            stats,
        )
        .add(color)
        .add(Recruitable(name.to_string()))
        .add(pos)
        .add(Direction::towards_center(pos, world))
        .build(),
    );
}

fn aftik_builder(display_info: DisplayInfo, name_data: NameData, stats: Stats) -> EntityBuilder {
    let mut builder = EntityBuilder::new();
    builder.add_bundle((
        display_info,
        name_data,
        Health::with_max(&stats),
        Stamina::with_max(&stats),
        stats,
    ));
    builder
}

pub fn place_goblin(world: &mut World, pos: Pos) {
    let stats = Stats::new(2, 4, 10);
    world.spawn((
        DisplayInfo::new('G', TextureType::Goblin, 10),
        NameData::from_noun("goblin", "goblins"),
        pos,
        Direction::towards_center(pos, world),
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
        DisplayInfo::new('E', TextureType::Eyesaur, 10),
        NameData::from_noun("eyesaur", "eyesaurs"),
        pos,
        Direction::towards_center(pos, world),
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
        DisplayInfo::new('Z', TextureType::Azureclops, 10),
        NameData::from_noun("azureclops", "azureclopses"),
        pos,
        Direction::towards_center(pos, world),
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
    color: AftikColor,
) -> Result<(), String> {
    let stock = shop_items
        .iter()
        .map(|item| to_priced_item(*item))
        .collect::<Result<Vec<_>, String>>()?;
    world.spawn((
        DisplayInfo::new('S', TextureType::Aftik, 15),
        color,
        NameData::from_noun("shopkeeper", "shopkeepers"),
        pos,
        Direction::towards_center(pos, world),
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
                item.noun_data().singular()
            )
        })
}
