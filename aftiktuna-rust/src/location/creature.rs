use crate::core::name::{Name, Noun};
use crate::core::position::{Direction, MovementBlocking, Pos};
use crate::core::status::{Health, Stamina, Stats};
use crate::core::{
    item, AftikColorId, CrewMember, Hostile, ModelId, OrderWeight, Recruitable, Shopkeeper,
    StockQuantity, StoreStock, Symbol,
};
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
    pub fn spawn(
        self,
        world: &mut World,
        symbol: Symbol,
        pos: Pos,
        health: f32,
        direction: Option<Direction>,
    ) {
        let health = Health::from_fraction(health);
        let is_alive = health.is_alive();
        let direction = direction.unwrap_or_else(|| Direction::towards_center(pos, world));
        let stats = self.default_stats();

        let mut builder = EntityBuilder::new();
        builder.add_bundle((
            symbol,
            OrderWeight::Creature,
            pos,
            direction,
            health,
            Stamina::with_max(&stats),
            stats,
        ));

        builder.add_bundle(match self {
            Type::Goblin => (ModelId::creature("goblin"), Noun::new("goblin", "goblins")),
            Type::Eyesaur => (
                ModelId::creature("eyesaur"),
                Noun::new("eyesaur", "eyesaurs"),
            ),
            Type::Azureclops => (
                ModelId::creature("azureclops"),
                Noun::new("azureclops", "azureclopses"),
            ),
            Type::Scarvie => (
                ModelId::creature("scarvie"),
                Noun::new("scarvie", "scarvies"),
            ),
            Type::VoraciousFrog => (
                ModelId::creature("voracious_frog"),
                Noun::new("voracious frog", "voracious frogs"),
            ),
        });

        if is_alive {
            builder.add_bundle(match self {
                Type::Goblin => (MovementBlocking, Hostile { aggressive: false }),
                Type::Eyesaur => (MovementBlocking, Hostile { aggressive: false }),
                Type::Azureclops => (MovementBlocking, Hostile { aggressive: true }),
                Type::Scarvie => (MovementBlocking, Hostile { aggressive: false }),
                Type::VoraciousFrog => (MovementBlocking, Hostile { aggressive: true }),
            });
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
    color: AftikColorId,
) -> Entity {
    world.spawn(
        aftik_builder(color, stats)
            .add(Name::known(name))
            .add(CrewMember(crew))
            .build(),
    )
}

pub fn place_recruitable(
    world: &mut World,
    pos: Pos,
    name: &str,
    stats: Stats,
    color: AftikColorId,
    direction: Option<Direction>,
) {
    let direction = direction.unwrap_or_else(|| Direction::towards_center(pos, world));

    world.spawn(
        aftik_builder(color, stats)
            .add(Name::not_known(name))
            .add(Recruitable)
            .add(pos)
            .add(direction)
            .build(),
    );
}

pub fn place_aftik_corpse(
    world: &mut World,
    pos: Pos,
    color: AftikColorId,
    direction: Option<Direction>,
) {
    let direction = direction.unwrap_or_else(|| Direction::towards_center(pos, world));

    world.spawn((
        ModelId::aftik(),
        OrderWeight::Creature,
        Noun::new("aftik", "aftiks"),
        Health::from_fraction(0.),
        color,
        pos,
        direction,
    ));
}

fn aftik_builder(color: AftikColorId, stats: Stats) -> EntityBuilder {
    let mut builder = EntityBuilder::new();
    builder.add_bundle((
        ModelId::aftik(),
        color,
        OrderWeight::Creature,
        Noun::new("aftik", "aftiks"),
        Health::from_fraction(1.),
        Stamina::with_max(&stats),
        stats,
    ));
    builder
}

pub fn place_shopkeeper(
    world: &mut World,
    pos: Pos,
    shop_stock: &[StockDefinition],
    color: AftikColorId,
    direction: Option<Direction>,
) -> Result<(), String> {
    let direction = direction.unwrap_or_else(|| Direction::towards_center(pos, world));
    let stock = shop_stock
        .iter()
        .map(StockDefinition::build)
        .collect::<Result<Vec<_>, String>>()?;
    world.spawn((
        ModelId::aftik(),
        OrderWeight::Creature,
        color,
        Noun::new("shopkeeper", "shopkeepers"),
        pos,
        direction,
        Shopkeeper(stock),
    ));
    Ok(())
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StockDefinition {
    item: item::Type,
    #[serde(default)]
    price: Option<item::Price>,
    #[serde(default)]
    quantity: Option<StockQuantity>,
}

impl StockDefinition {
    fn build(&self) -> Result<StoreStock, String> {
        let Self {
            item,
            price,
            quantity,
        } = *self;
        let price = price.or_else(|| item.price()).ok_or_else(|| {
            format!(
                "Cannot get a price from item \"{}\" to put in store",
                item.noun_data().singular()
            )
        })?;
        let quantity = quantity.unwrap_or(StockQuantity::Unlimited);
        Ok(StoreStock {
            item,
            price,
            quantity,
        })
    }
}
