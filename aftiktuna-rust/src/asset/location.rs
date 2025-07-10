pub mod creature {
    use crate::asset::ProfileOrRandom;
    use crate::core::display::{AftikColorId, ModelId};
    use crate::core::name::Noun;
    use crate::core::position::Direction;
    use crate::core::status::Stats;
    use crate::core::store::StockQuantity;
    use crate::core::{CreatureAttribute, GivesHuntReward, Tag, item};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum AttributeChoice {
        None,
        #[default]
        Random,
        #[serde(untagged)]
        Attribute(CreatureAttribute),
    }

    impl AttributeChoice {
        pub fn variants() -> impl Iterator<Item = Self> {
            [Self::None, Self::Random].into_iter().chain(
                CreatureAttribute::variants()
                    .iter()
                    .copied()
                    .map(Self::Attribute),
            )
        }
    }

    #[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Type {
        Goblin,
        Eyesaur,
        Azureclops,
        Scarvie,
        VoraciousFrog,
    }

    impl Type {
        pub fn variants() -> &'static [Self] {
            use Type::*;
            &[Goblin, Eyesaur, Azureclops, Scarvie, VoraciousFrog]
        }

        pub fn is_aggressive_by_default(self) -> bool {
            match self {
                Self::Goblin | Self::Eyesaur | Self::Scarvie => false,
                Self::Azureclops | Self::VoraciousFrog => true,
            }
        }

        pub fn default_stats(self) -> Stats {
            match self {
                Self::Goblin => Stats::new(2, 4, 10, 2),
                Self::Eyesaur => Stats::new(7, 7, 4, 2),
                Self::Azureclops => Stats::new(15, 10, 4, 2),
                Self::Scarvie => Stats::new(3, 2, 8, 1),
                Self::VoraciousFrog => Stats::new(8, 8, 3, 3),
            }
        }

        pub fn model_id(self) -> ModelId {
            ModelId::creature(match self {
                Self::Goblin => "goblin",
                Self::Eyesaur => "eyesaur",
                Self::Azureclops => "azureclops",
                Self::Scarvie => "scarvie",
                Self::VoraciousFrog => "voracious_frog",
            })
        }

        pub fn noun(self) -> Noun {
            match self {
                Self::Goblin => Noun::new("goblin", "goblins"),
                Self::Eyesaur => Noun::new("eyesaur", "eyesaurs"),
                Self::Azureclops => Noun::new("azureclops", "azureclopses"),
                Self::Scarvie => Noun::new("scarvie", "scarvies"),
                Self::VoraciousFrog => Noun::new("voracious frog", "voracious frogs"),
            }
        }
    }

    #[derive(Clone, Serialize, Deserialize)]
    pub struct CreatureSpawnData {
        pub creature: Type,
        #[serde(default = "full_health", skip_serializing_if = "is_full_health")]
        pub health: f32,
        #[serde(default, skip_serializing_if = "crate::is_default")]
        pub attribute: AttributeChoice,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub aggressive: Option<bool>,
        #[serde(default, skip_serializing_if = "crate::is_default")]
        pub wandering: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub tag: Option<Tag>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub direction: Option<Direction>,
    }

    fn full_health() -> f32 {
        1.
    }

    fn is_full_health(health: &f32) -> bool {
        *health == 1.
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum CharacterInteraction {
        Recruitable,
        GivesHuntReward(GivesHuntReward),
    }

    #[derive(Clone, Serialize, Deserialize)]
    pub struct NpcSpawnData {
        #[serde(default, skip_serializing_if = "ProfileOrRandom::is_default")]
        pub profile: ProfileOrRandom,
        pub interaction: CharacterInteraction,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub direction: Option<Direction>,
    }

    #[derive(Clone, Serialize, Deserialize)]
    pub struct AftikCorpseData {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub color: Option<AftikColorId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub direction: Option<Direction>,
    }

    #[derive(Clone, Serialize, Deserialize)]
    pub struct ShopkeeperSpawnData {
        pub stock: Vec<StockDefinition>,
        pub color: AftikColorId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub direction: Option<Direction>,
    }

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub struct StockDefinition {
        pub item: item::Type,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub price: Option<item::Price>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub quantity: Option<StockQuantity>,
    }
}

use crate::asset::loot::{self, LootTableId};
use crate::core::area::BackgroundId;
use crate::core::display::ModelId;
use crate::core::name::Noun;
use crate::core::position::Direction;
use crate::core::{BlockType, DoorKind, item};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fs::File;

pub type SymbolMap = IndexMap<char, SymbolData>;
pub type DoorPairMap = IndexMap<String, DoorPairData>;

#[derive(Serialize, Deserialize)]
pub struct LocationData {
    pub areas: Vec<AreaData>,
    pub door_pairs: DoorPairMap,
}

impl LocationData {
    pub fn load_from_json(name: &str) -> Result<Self, String> {
        super::load_json_simple(format!("location/{name}.json"))
    }
}

#[derive(Serialize, Deserialize)]
pub struct AreaData {
    pub name: String,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub pos_in_overview: (i32, i32),
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub background: BackgroundId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_offset: Option<i32>,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub darkness: f32,
    pub objects: Vec<String>,
    pub symbols: SymbolMap,
}

pub fn load_base_symbols() -> Result<SymbolMap, String> {
    let file = File::open("assets/symbols.json")
        .map_err(|error| format!("Failed to open symbols file: {error}"))?;
    serde_json::from_reader::<_, SymbolMap>(file)
        .map_err(|error| format!("Failed to parse symbols file: {error}"))
}

pub struct SymbolLookup<'a> {
    parent_map: &'a SymbolMap,
    map: &'a SymbolMap,
}

impl<'a> SymbolLookup<'a> {
    pub fn new(parent_map: &'a SymbolMap, map: &'a SymbolMap) -> Self {
        Self { parent_map, map }
    }

    pub fn lookup(&self, symbol: char) -> Option<&'a SymbolData> {
        self.map
            .get(&symbol)
            .or_else(|| self.parent_map.get(&symbol))
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SymbolData {
    LocationEntry,
    FortunaChest,
    Item {
        item: item::Type,
    },
    Loot {
        table: loot::LootTableId,
    },
    Door(DoorSpawnData),
    Inanimate {
        model: ModelId,
        #[serde(default, skip_serializing_if = "crate::is_default")]
        direction: Direction,
    },
    Container(ContainerData),
    Creature(creature::CreatureSpawnData),
    Shopkeeper(creature::ShopkeeperSpawnData),
    Character(creature::NpcSpawnData),
    AftikCorpse(creature::AftikCorpseData),
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DoorPairData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_type: Option<BlockType>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoorType {
    Door,
    Doorway,
    Shack,
    House,
    Store,
    Path,
    LeftPath,
    RightPath,
    CrossroadPath,
}

impl From<DoorType> for ModelId {
    fn from(value: DoorType) -> Self {
        match value {
            DoorType::Door => ModelId::new("door"),
            DoorType::Doorway => ModelId::new("doorway"),
            DoorType::Shack | DoorType::House | DoorType::Store => ModelId::new("shack"),
            DoorType::Path => ModelId::new("path"),
            DoorType::LeftPath => ModelId::new("path/left_corner"),
            DoorType::RightPath => ModelId::new("path/right_corner"),
            DoorType::CrossroadPath => ModelId::new("path/crossroad"),
        }
    }
}

impl From<DoorType> for DoorKind {
    fn from(value: DoorType) -> Self {
        match value {
            DoorType::Door
            | DoorType::Doorway
            | DoorType::Shack
            | DoorType::House
            | DoorType::Store => DoorKind::Door,
            DoorType::Path | DoorType::LeftPath | DoorType::RightPath | DoorType::CrossroadPath => {
                DoorKind::Path
            }
        }
    }
}

impl DoorType {
    pub fn variants() -> &'static [Self] {
        use DoorType::*;
        &[
            Door,
            Doorway,
            Shack,
            House,
            Store,
            Path,
            LeftPath,
            RightPath,
            CrossroadPath,
        ]
    }

    pub fn noun(self, adjective: Option<DoorAdjective>) -> Noun {
        let noun = match self {
            Self::Door => Noun::new("door", "doors"),
            Self::Doorway => Noun::new("doorway", "doorways"),
            Self::Shack => Noun::new("shack", "shacks"),
            Self::House => Noun::new("house", "houses"),
            Self::Store => Noun::new("store", "stores"),
            Self::Path | Self::LeftPath | Self::RightPath | Self::CrossroadPath => {
                Noun::new("path", "paths")
            }
        };
        if let Some(adjective) = adjective {
            noun.with_adjective(adjective.word())
        } else {
            noun
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoorAdjective {
    Left,
    Middle,
    Right,
}

impl DoorAdjective {
    pub fn variants() -> &'static [Self] {
        use DoorAdjective::*;
        &[Left, Middle, Right]
    }

    pub fn word(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Middle => "middle",
            Self::Right => "right",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DoorSpawnData {
    pub pair_id: String,
    pub display_type: DoorType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adjective: Option<DoorAdjective>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContainerType {
    Tent,
    Cabinet,
    Drawer,
    Crate,
    Chest,
    CrashedShip,
}

impl ContainerType {
    pub fn variants() -> &'static [Self] {
        use ContainerType::*;
        &[Tent, Cabinet, Drawer, Crate, Chest, CrashedShip]
    }

    pub fn model_id(self) -> ModelId {
        ModelId::new(match self {
            Self::Tent => "container/tent",
            Self::Cabinet => "container/cabinet",
            Self::Drawer => "container/drawer",
            Self::Crate => "container/crate",
            Self::Chest => "container/chest",
            Self::CrashedShip => "container/crashed_ship",
        })
    }

    pub fn noun(self) -> Noun {
        match self {
            Self::Tent => Noun::new("tent", "tents"),
            Self::Cabinet => Noun::new("cabinet", "cabinets"),
            Self::Drawer => Noun::new("drawer", "drawers"),
            Self::Crate => Noun::new("crate", "crates"),
            Self::Chest => Noun::new("chest", "chests"),
            Self::CrashedShip => Noun::new("crashed ship", "crashed ships"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ItemOrLoot {
    Item { item: item::Type },
    Loot { table: LootTableId },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerData {
    pub container_type: ContainerType,
    pub content: Vec<ItemOrLoot>,
    #[serde(default)]
    pub direction: Direction,
}
