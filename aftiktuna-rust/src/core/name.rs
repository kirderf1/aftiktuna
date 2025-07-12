use hecs::{Entity, EntityRef, World};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;

use super::CreatureAttribute;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum NameData {
    Name(String),
    Noun(Noun),
}

impl NameData {
    pub fn base(&self) -> &str {
        match self {
            NameData::Name(name) => name,
            NameData::Noun(noun) => &noun.singular,
        }
    }
    pub fn plural(&self) -> &str {
        match self {
            NameData::Name(name) => name,
            NameData::Noun(noun) => &noun.plural,
        }
    }
    pub fn definite(&self) -> String {
        match self {
            NameData::Name(name) => name.to_string(),
            NameData::Noun(noun) => format!("the {}", noun.singular),
        }
    }

    pub fn find_option_by_ref(entity_ref: EntityRef) -> Option<Self> {
        if let Some(name) = entity_ref.get::<&Name>() {
            if name.is_known {
                return Some(Self::Name(name.name.clone()));
            }
        }
        entity_ref
            .get::<&Noun>()
            .map(|noun| NameData::Noun(noun.deref().clone()))
    }

    pub fn find_by_ref(entity_ref: EntityRef) -> Self {
        Self::find_option_by_ref(entity_ref).unwrap_or_default()
    }

    pub fn find(world: &World, entity: Entity) -> Self {
        world
            .entity(entity)
            .ok()
            .and_then(Self::find_option_by_ref)
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Default)]
pub struct NameWithAttribute(NameData, Option<CreatureAttribute>);

impl NameWithAttribute {
    pub fn lookup_by_ref(entity_ref: EntityRef) -> Self {
        Self::lookup_option_by_ref(entity_ref).unwrap_or_default()
    }

    pub fn lookup_option_by_ref(entity_ref: EntityRef) -> Option<Self> {
        Some(Self(
            NameData::find_option_by_ref(entity_ref)?,
            entity_ref.get::<&CreatureAttribute>().as_deref().copied(),
        ))
    }

    pub fn lookup(entity: Entity, world: &World) -> Self {
        world
            .entity(entity)
            .ok()
            .map(Self::lookup_by_ref)
            .unwrap_or_default()
    }

    pub fn base(&self) -> String {
        match (&self.0, self.1) {
            (NameData::Name(name), _) => name.to_owned(),
            (NameData::Noun(noun), Some(attribute)) => {
                format!(
                    "{adjective} {entity}",
                    adjective = attribute.as_adjective(),
                    entity = noun.singular
                )
            }
            (NameData::Noun(noun), None) => noun.singular.to_owned(),
        }
    }

    pub fn definite(&self) -> String {
        match (&self.0, self.1) {
            (NameData::Name(name), _) => name.to_owned(),
            (NameData::Noun(noun), Some(attribute)) => {
                format!(
                    "the {adjective} {entity}",
                    adjective = attribute.as_adjective(),
                    entity = noun.singular
                )
            }
            (NameData::Noun(noun), None) => format!("the {entity}", entity = noun.singular),
        }
    }
}

impl Default for NameData {
    fn default() -> Self {
        Self::Name("???".to_owned())
    }
}

impl<'a> From<NameQuery<'a>> for NameData {
    fn from(query: NameQuery<'a>) -> Self {
        let (name, noun) = query;
        if let Some(name) = name {
            if name.is_known {
                return Self::Name(name.name.clone());
            }
        }
        noun.map(|noun| NameData::Noun(noun.clone()))
            .unwrap_or_default()
    }
}

pub type NameQuery<'a> = (Option<&'a Name>, Option<&'a Noun>);

#[derive(Debug, Serialize, Deserialize)]
pub struct Name {
    pub name: String,
    pub is_known: bool,
}

impl Name {
    pub fn known(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            is_known: true,
        }
    }
    pub fn not_known(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            is_known: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Noun {
    singular: String,
    plural: String,
}

impl Noun {
    pub fn new(singular: &str, plural: &str) -> Self {
        Self {
            singular: singular.to_string(),
            plural: plural.to_string(),
        }
    }

    pub fn singular(&self) -> &str {
        &self.singular
    }

    pub fn plural(&self) -> &str {
        &self.plural
    }

    pub fn with_adjective(&self, adjective: &str) -> Noun {
        Noun {
            singular: format!("{} {}", adjective, self.singular),
            plural: format!("{} {}", adjective, self.plural),
        }
    }

    pub fn for_count(&self, count: u16) -> &str {
        if count == 1 {
            &self.singular
        } else {
            &self.plural
        }
    }

    pub fn with_text_count(&self, count: u16, article: Article) -> String {
        self.with_count(count, article, CountFormat::Text)
    }

    pub fn with_count(&self, count: u16, article: Article, format: CountFormat) -> String {
        if article == Article::The {
            if count == 1 {
                format!("the {name}", name = self.singular(),)
            } else {
                format!(
                    "the {count} {name}",
                    count = format.apply(count),
                    name = self.for_count(count),
                )
            }
        } else if article == Article::A && count == 1 {
            format!("a {name}", name = self.singular(),)
        } else {
            format!(
                "{count} {name}",
                count = format.apply(count),
                name = self.for_count(count),
            )
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountFormat {
    Numeric,
    Text,
}

impl CountFormat {
    fn apply(self, count: u16) -> String {
        if self == Self::Numeric {
            return count.to_string();
        }
        match count {
            1 => "one",
            2 => "two",
            3 => "three",
            4 => "four",
            5 => "five",
            6 => "six",
            7 => "seven",
            8 => "eight",
            9 => "nine",
            10 => "ten",
            _ => return count.to_string(),
        }
        .to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Article {
    The,
    A,
    One,
}

pub fn names_with_counts(
    data: impl IntoIterator<Item = NameData>,
    article: Article,
    format: CountFormat,
) -> Vec<String> {
    let mut names = Vec::new();
    let mut nouns = HashMap::new();

    for name_data in data {
        match name_data {
            NameData::Name(name) => names.push(name),
            NameData::Noun(noun) => *nouns.entry(noun).or_insert(0) += 1,
        }
    }

    names
        .into_iter()
        .chain(
            nouns
                .into_iter()
                .map(|(noun, count)| noun.with_count(count, article, format)),
        )
        .collect::<Vec<String>>()
}
