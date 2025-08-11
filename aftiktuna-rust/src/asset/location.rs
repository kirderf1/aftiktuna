pub mod creature {
    use crate::asset::ProfileOrRandom;
    use crate::core::display::{AftikColorId, ModelId};
    use crate::core::item::{self, ItemType};
    use crate::core::name::{IndefiniteArticle, Noun};
    use crate::core::position::Direction;
    use crate::core::status::Stats;
    use crate::core::store::StockQuantity;
    use crate::core::{AttackSet, CreatureAttribute, GivesHuntReward, Tag, UnarmedType};
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
        BloodMantis,
    }

    impl Type {
        pub fn variants() -> &'static [Self] {
            use Type::*;
            &[
                Goblin,
                Eyesaur,
                Azureclops,
                Scarvie,
                VoraciousFrog,
                BloodMantis,
            ]
        }

        pub fn is_aggressive_by_default(self) -> bool {
            match self {
                Self::Goblin | Self::Eyesaur | Self::Scarvie => false,
                Self::Azureclops | Self::VoraciousFrog | Self::BloodMantis => true,
            }
        }

        pub fn is_tameable(self) -> bool {
            matches!(self, Self::Eyesaur | Self::Scarvie)
        }

        pub fn default_stats(self) -> Stats {
            match self {
                Self::Goblin => Stats::new(2, 4, 10, 2),
                Self::Eyesaur => Stats::new(7, 7, 4, 2),
                Self::Azureclops => Stats::new(15, 10, 4, 2),
                Self::Scarvie => Stats::new(3, 2, 8, 1),
                Self::VoraciousFrog => Stats::new(8, 8, 3, 3),
                Self::BloodMantis => Stats::new(15, 5, 10, 5),
            }
        }

        pub fn model_id(self) -> ModelId {
            ModelId::creature(match self {
                Self::Goblin => "goblin",
                Self::Eyesaur => "eyesaur",
                Self::Azureclops => "azureclops",
                Self::Scarvie => "scarvie",
                Self::VoraciousFrog => "voracious_frog",
                Self::BloodMantis => "blood_mantis",
            })
        }

        pub fn noun(self) -> Noun {
            use IndefiniteArticle::*;
            match self {
                Self::Goblin => Noun::new("goblin", "goblins", A),
                Self::Eyesaur => Noun::new("eyesaur", "eyesaurs", An),
                Self::Azureclops => Noun::new("azureclops", "azureclopses", An),
                Self::Scarvie => Noun::new("scarvie", "scarvies", A),
                Self::VoraciousFrog => Noun::new("voracious frog", "voracious frogs", A),
                Self::BloodMantis => Noun::new("blood mantis", "blood mantes", A),
            }
        }

        pub fn unarmed_type(self) -> UnarmedType {
            match self {
                Self::Goblin => UnarmedType::Scratch,
                Self::Eyesaur => UnarmedType::Bite,
                Self::Azureclops => UnarmedType::Punch,
                Self::Scarvie => UnarmedType::Bite,
                Self::VoraciousFrog => UnarmedType::Pounce,
                Self::BloodMantis => UnarmedType::Slash,
            }
        }

        pub fn attack_set(self) -> AttackSet {
            match self {
                Self::Goblin => AttackSet::Light,
                Self::Eyesaur => AttackSet::Quick,
                Self::Azureclops => AttackSet::Varied,
                Self::Scarvie => AttackSet::Light,
                Self::VoraciousFrog => AttackSet::Slow,
                Self::BloodMantis => AttackSet::Quick,
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
        pub item: ItemType,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub price: Option<item::Price>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub quantity: Option<StockQuantity>,
    }
}

use crate::asset::background::ParallaxLayer;
use crate::asset::loot::{self, LootTableId};
use crate::core::area::BackgroundId;
use crate::core::display::ModelId;
use crate::core::item::ItemType;
use crate::core::name::{IndefiniteArticle, Noun};
use crate::core::position::Direction;
use crate::core::{BlockType, DoorKind};
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_background_layers: Vec<ParallaxLayer<String>>,
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
    ShipControls {
        direction: Direction,
    },
    FoodDeposit,
    Item {
        item: ItemType,
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
    Furnish {
        template: String,
    },
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DoorPairData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_type: Option<BlockType>,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub is_cut: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoorType {
    Door,
    Doorway,
    UpwardStairs,
    DownwardStairs,
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
        ModelId::new(match value {
            DoorType::Door | DoorType::House | DoorType::Store => "door/door",
            DoorType::Doorway => "door/doorway",
            DoorType::UpwardStairs => "door/upward_stairs",
            DoorType::DownwardStairs => "door/downward_stairs",
            DoorType::Shack => "door/shack",
            DoorType::Path => "path",
            DoorType::LeftPath => "path/left_corner",
            DoorType::RightPath => "path/right_corner",
            DoorType::CrossroadPath => "path/crossroad",
        })
    }
}

impl From<DoorType> for DoorKind {
    fn from(value: DoorType) -> Self {
        match value {
            DoorType::Door
            | DoorType::Doorway
            | DoorType::UpwardStairs
            | DoorType::DownwardStairs
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
            UpwardStairs,
            DownwardStairs,
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
            Self::Door => Noun::new("door", "doors", IndefiniteArticle::A),
            Self::Doorway => Noun::new("doorway", "doorways", IndefiniteArticle::A),
            Self::UpwardStairs => {
                Noun::new("upward stairs", "upward stairs", IndefiniteArticle::An)
            }
            Self::DownwardStairs => {
                Noun::new("downward stairs", "downward stairs", IndefiniteArticle::A)
            }
            Self::Shack => Noun::new("shack", "shacks", IndefiniteArticle::A),
            Self::House => Noun::new("house", "houses", IndefiniteArticle::A),
            Self::Store => Noun::new("store", "stores", IndefiniteArticle::A),
            Self::Path | Self::LeftPath | Self::RightPath | Self::CrossroadPath => {
                Noun::new("path", "paths", IndefiniteArticle::A)
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
            Self::Tent => Noun::new("tent", "tents", IndefiniteArticle::A),
            Self::Cabinet => Noun::new("cabinet", "cabinets", IndefiniteArticle::A),
            Self::Drawer => Noun::new("drawer", "drawers", IndefiniteArticle::A),
            Self::Crate => Noun::new("crate", "crates", IndefiniteArticle::A),
            Self::Chest => Noun::new("chest", "chests", IndefiniteArticle::A),
            Self::CrashedShip => Noun::new("crashed ship", "crashed ships", IndefiniteArticle::A),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ItemOrLoot {
    Item { item: ItemType },
    Loot { table: LootTableId },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerData {
    pub container_type: ContainerType,
    pub content: Vec<ItemOrLoot>,
    #[serde(default)]
    pub direction: Direction,
}

#[derive(Serialize, Deserialize)]
pub struct FurnishTemplate {
    pub objects: Vec<String>,
    pub symbols: SymbolMap,
}

impl FurnishTemplate {
    pub fn load_list(template: &str) -> Result<Vec<Self>, String> {
        super::load_json_simple(format!("area_furnish/{template}.json"))
    }
}
