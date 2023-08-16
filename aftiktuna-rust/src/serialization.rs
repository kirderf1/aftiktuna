use crate::action::combat::IsFoe;
use crate::action::door::{BlockType, Door};
use crate::action::item::Held;
use crate::action::trade::{IsTrading, Points, Shopkeeper};
use crate::action::{Action, CrewMember, FortunaChest, OpenedChest, Recruitable};
use crate::ai::Intention;
use crate::area::{Area, Ship};
use crate::game_loop::{Game, GameState};
use crate::item::{Blowtorch, CanWield, Crowbar, FuelCan, Item, Keycard, Medkit, Price, Weapon};
use crate::position::{Direction, MovementBlocking, Pos};
use crate::status::{Health, LowHealth, LowStamina, Stamina, Stats};
use crate::view::{AftikColor, DisplayInfo, NameData};
use hecs::serialize::column;
use hecs::{Archetype, ColumnBatchBuilder, ColumnBatchType, World};
use rmp_serde::{decode, encode};
use serde::de::SeqAccess;
use serde::ser::SerializeTuple;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::any::TypeId;
use std::io::{Read, Write};

macro_rules! components_to_serialize {
    ($($comp:ty, $id:ident);+ $(;)?) => {
        #[derive(Copy, Clone, Serialize, Deserialize)]
        enum ComponentId {
            $(
            $id
            ),*
        }
        impl column::DeserializeContext for HecsDeserializeContext {
            fn deserialize_component_ids<'de, A>(
                &mut self,
                mut seq: A,
            ) -> Result<ColumnBatchType, A::Error>
            where
                A: SeqAccess<'de>,
            {
                self.0.clear();
                let mut batch = ColumnBatchType::new();
                while let Some(id) = seq.next_element()? {
                    match id {
                        $(
                        ComponentId::$id => {
                            batch.add::<$comp>();
                        }
                        )*
                    }
                    self.0.push(id);
                }
                Ok(batch)
            }

            fn deserialize_components<'de, A>(
                &mut self,
                entity_count: u32,
                mut seq: A,
                batch: &mut ColumnBatchBuilder,
            ) -> Result<(), A::Error>
            where
                A: SeqAccess<'de>,
            {
                for &id in &self.0 {
                    match id {
                        $(
                        ComponentId::$id => {
                            column::deserialize_column::<$comp, _>(entity_count, &mut seq, batch)?;
                        }
                        )*
                    }
                }
                Ok(())
            }
        }
        fn is_serialized_type_id(id: TypeId) -> bool {
            $(id == TypeId::of::<$comp>())||*
        }
        impl column::SerializeContext for HecsSerializeContext {
            fn component_count(&self, archetype: &Archetype) -> usize {
                archetype
                    .component_types()
                    .filter(|&id| is_serialized_type_id(id))
                    .count()
            }

            fn serialize_component_ids<S: SerializeTuple>(
                &mut self,
                archetype: &Archetype,
                mut out: S,
            ) -> Result<S::Ok, S::Error> {
                $(
                column::try_serialize_id::<$comp, _, _>(archetype, &ComponentId::$id, &mut out)?;
                )*
                out.end()
            }

            fn serialize_components<S: SerializeTuple>(
                &mut self,
                archetype: &Archetype,
                mut out: S,
            ) -> Result<S::Ok, S::Error> {
                $(
                column::try_serialize::<$comp, _>(archetype, &mut out)?;
                )*
                out.end()
            }
        }
    };
}

struct HecsSerializeContext;

#[derive(Default)]
struct HecsDeserializeContext(Vec<ComponentId>);

components_to_serialize!(
    Area, Area;
    Ship, Ship;
    Pos, Pos;
    Direction, Direction;
    MovementBlocking, MovementBlocking;

    NameData, NameData;
    DisplayInfo, DisplayInfo;
    AftikColor, AftikColor;

    Stats, Stats;
    Health, Health;
    Stamina, Stamina;
    LowHealth, LowHealth;
    LowStamina, LowStamina;

    CrewMember, CrewMember;
    IsFoe, IsFoe;
    Action, Action;
    Intention, Intention;

    Recruitable, Recruitable;
    Shopkeeper, Shopkeeper;
    IsTrading, IsTrading;
    Points, Points;

    Door, Door;
    BlockType, BlockType;

    Held, Held;
    Item, Item;
    FuelCan, FuelCan;
    Medkit, Medkit;
    Crowbar, Crowbar;
    Blowtorch, Blowtorch;
    Keycard, Keycard;
    CanWield, CanWield;
    Weapon, Weapon;
    Price, Price;

    FortunaChest, FortunaChest;
    OpenedChest, OpenedChest;
);

#[derive(Serialize)]
struct SerializedData<'a> {
    world: SerializedWorld<'a>,
    state: &'a GameState,
}

impl<'a> From<&'a Game> for SerializedData<'a> {
    fn from(value: &'a Game) -> Self {
        Self {
            world: SerializedWorld(&value.world),
            state: &value.state,
        }
    }
}

#[derive(Deserialize)]
struct DeserializedData {
    world: DeserializedWorld,
    state: GameState,
}

impl From<DeserializedData> for Game {
    fn from(value: DeserializedData) -> Self {
        Self::new(value.world.0, value.state)
    }
}

struct SerializedWorld<'a>(&'a World);

impl<'a> Serialize for SerializedWorld<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        column::serialize(self.0, &mut HecsSerializeContext, serializer)
    }
}

struct DeserializedWorld(World);

impl<'de> Deserialize<'de> for DeserializedWorld {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        column::deserialize(&mut HecsDeserializeContext::default(), deserializer)
            .map(DeserializedWorld)
    }
}

pub fn serialize_game(game: &Game, writer: impl Write) -> Result<(), encode::Error> {
    let mut serializer = rmp_serde::Serializer::new(writer).with_struct_map();
    SerializedData::from(game).serialize(&mut serializer)
}

pub fn deserialize_game(reader: impl Read) -> Result<Game, decode::Error> {
    let mut deserializer = rmp_serde::Deserializer::new(reader);
    DeserializedData::deserialize(&mut deserializer).map(Game::from)
}
