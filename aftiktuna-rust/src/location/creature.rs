use crate::action::trade::{PricedItem, Shopkeeper};
use crate::core::ai::IsFoe;
use crate::core::position::{Direction, MovementBlocking, Pos};
use crate::core::status::{Health, Stamina, Stats};
use crate::core::{item, CrewMember, Recruitable};
use crate::view::area::{AftikColor, OrderWeight, Symbol, TextureType};
use crate::view::name::{Name, Noun};
use hecs::{Entity, EntityBuilder, World};
use serde::{Deserialize, Serialize};

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
    pub fn spawn(self, world: &mut World, symbol: Symbol, pos: Pos, direction: Option<Direction>) {
        let direction = direction.unwrap_or_else(|| Direction::towards_center(pos, world));
        let stats = self.default_stats();
        let mut builder = EntityBuilder::new();
        builder.add_bundle((
            symbol,
            OrderWeight::Creature,
            pos,
            direction,
            Health::with_max(&stats),
            Stamina::with_max(&stats),
            stats,
        ));
        match self {
            Type::Goblin => {
                builder.add_bundle((
                    TextureType::creature("goblin"),
                    Noun::new("goblin", "goblins"),
                    MovementBlocking,
                    IsFoe,
                ));
            }
            Type::Eyesaur => {
                builder.add_bundle((
                    TextureType::creature("eyesaur"),
                    Noun::new("eyesaur", "eyesaurs"),
                    MovementBlocking,
                    IsFoe,
                ));
            }
            Type::Azureclops => {
                builder.add_bundle((
                    TextureType::creature("azureclops"),
                    Noun::new("azureclops", "azureclopses"),
                    MovementBlocking,
                    IsFoe,
                ));
            }
            Type::Scarvie => {
                builder.add_bundle((
                    TextureType::creature("scarvie"),
                    Noun::new("scarvie", "scarvies"),
                    MovementBlocking,
                    IsFoe,
                ));
            }
            Type::VoraciousFrog => {
                builder.add_bundle((
                    TextureType::creature("voracious_frog"),
                    Noun::new("voracious frog", "voracious frogs"),
                    MovementBlocking,
                    IsFoe,
                ));
            }
        }
        world.spawn(builder.build());
    }

    fn default_stats(self) -> Stats {
        match self {
            Type::Goblin => Stats::new(2, 4, 10),
            Type::Eyesaur => Stats::new(7, 7, 4),
            Type::Azureclops => Stats::new(15, 10, 4),
            Type::Scarvie => Stats::new(3, 2, 8),
            Type::VoraciousFrog => Stats::new(8, 8, 3),
        }
    }
}

pub fn spawn_crew_member(
    world: &mut World,
    crew: Entity,
    name: &str,
    stats: Stats,
    color: AftikColor,
) -> Entity {
    world.spawn(
        aftik_builder(Symbol::from_name(name), Name::known(name), stats)
            .add(color)
            .add(CrewMember(crew))
            .build(),
    )
}

pub fn place_recruitable(
    world: &mut World,
    symbol: Symbol,
    pos: Pos,
    name: &str,
    stats: Stats,
    color: AftikColor,
    direction: Option<Direction>,
) {
    let direction = direction.unwrap_or_else(|| Direction::towards_center(pos, world));

    world.spawn(
        aftik_builder(symbol, Name::not_known(name), stats)
            .add(color)
            .add(Recruitable)
            .add(pos)
            .add(direction)
            .build(),
    );
}

fn aftik_builder(symbol: Symbol, name: Name, stats: Stats) -> EntityBuilder {
    let mut builder = EntityBuilder::new();
    builder.add_bundle((
        symbol,
        TextureType::creature("aftik"),
        OrderWeight::Creature,
        Noun::new("aftik", "aftiks"),
        name,
        Health::with_max(&stats),
        Stamina::with_max(&stats),
        stats,
    ));
    builder
}

pub fn place_shopkeeper(
    world: &mut World,
    symbol: Symbol,
    pos: Pos,
    shop_items: &[item::Type],
    color: AftikColor,
    direction: Option<Direction>,
) -> Result<(), String> {
    let direction = direction.unwrap_or_else(|| Direction::towards_center(pos, world));
    let stock = shop_items
        .iter()
        .map(|item| to_priced_item(*item))
        .collect::<Result<Vec<_>, String>>()?;
    world.spawn((
        symbol,
        TextureType::aftik(),
        OrderWeight::Creature,
        color,
        Noun::new("shopkeeper", "shopkeepers"),
        pos,
        direction,
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
