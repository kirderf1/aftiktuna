use hecs::{Entity, EntityRef, World};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;

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

    pub fn matches(&self, string: &str) -> bool {
        self.base().eq_ignore_ascii_case(string)
    }
    pub fn matches_plural(&self, string: &str) -> bool {
        self.plural().eq_ignore_ascii_case(string)
    }
    pub fn matches_with_count(&self, string: &str, count: u16) -> bool {
        match self {
            NameData::Name(name) => name,
            NameData::Noun(noun) => noun.for_count(count),
        }
        .eq_ignore_ascii_case(string)
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

    pub fn with_count(&self, count: u16) -> String {
        format!("{} {}", count, self.for_count(count))
    }
}

pub fn names_with_counts(data: impl IntoIterator<Item = NameData>) -> Vec<String> {
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
                .map(|(noun, count)| noun.with_count(count)),
        )
        .collect::<Vec<String>>()
}
