use crate::action::combat::IsFoe;
use crate::action::door::{BlockType, Door};
use crate::action::item::Held;
use crate::action::trade::{IsTrading, Points, Shopkeeper};
use crate::action::{Action, CrewMember, FortunaChest, OpenedChest, Recruitable};
use crate::ai::Intention;
use crate::area::{Area, Ship};
use crate::game_loop::{LowHealth, LowStamina};
use crate::item::{Blowtorch, CanWield, Crowbar, FuelCan, Item, Keycard, Medkit, Price, Weapon};
use crate::position::{Direction, MovementBlocking, Pos};
use crate::status::{Health, Stamina, Stats};
use crate::view::{AftikColor, DisplayInfo, NameData};
use hecs::serialize::column::{
    deserialize, deserialize_column, serialize, try_serialize, try_serialize_id,
    DeserializeContext, SerializeContext,
};
use hecs::{Archetype, ColumnBatchBuilder, ColumnBatchType, World};
use serde::de::SeqAccess;
use serde::ser::SerializeTuple;
use serde::{Deserialize, Serialize};
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
        impl DeserializeContext for OurDeserialize {
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
                            deserialize_column::<$comp, _>(entity_count, &mut seq, batch)?;
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
        impl SerializeContext for OurSerialize {
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
                try_serialize_id::<$comp, _, _>(archetype, &ComponentId::$id, &mut out)?;
                )*
                out.end()
            }

            fn serialize_components<S: SerializeTuple>(
                &mut self,
                archetype: &Archetype,
                mut out: S,
            ) -> Result<S::Ok, S::Error> {
                $(
                try_serialize::<$comp, _>(archetype, &mut out)?;
                )*
                out.end()
            }
        }
    };
}

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
    LowStamina, LowStamina;
    LowHealth, LowHealth;

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

#[derive(Default)]
struct OurDeserialize(Vec<ComponentId>);

struct OurSerialize;

pub fn serialize_world(world: &World, writer: impl Write) {
    let mut serializer = rmp_serde::Serializer::new(writer).with_struct_map();
    serialize(world, &mut OurSerialize, &mut serializer).unwrap();
}

pub fn deserialize_world(reader: impl Read) -> World {
    let mut deserializer = rmp_serde::Deserializer::new(reader);
    deserialize(&mut OurDeserialize::default(), &mut deserializer).unwrap()
}
