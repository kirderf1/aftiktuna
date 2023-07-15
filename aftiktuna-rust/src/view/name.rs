use hecs::{Entity, World};
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Data {
    Name(String),
    Noun(NounData),
}

impl Data {
    pub fn from_name(name: &str) -> Self {
        Data::Name(name.to_string())
    }
    pub fn from_noun(singular: &str, plural: &str) -> Self {
        Data::Noun(NounData::new(singular, plural))
    }

    pub fn base(&self) -> &str {
        match self {
            Data::Name(name) => name,
            Data::Noun(noun) => &noun.singular,
        }
    }
    pub fn plural(&self) -> &str {
        match self {
            Data::Name(name) => name,
            Data::Noun(noun) => &noun.plural,
        }
    }
    pub fn definite(&self) -> String {
        match self {
            Data::Name(name) => name.to_string(),
            Data::Noun(noun) => format!("the {}", noun.singular),
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
            Data::Name(name) => name,
            Data::Noun(noun) => noun.for_count(count),
        }
        .eq_ignore_ascii_case(string)
    }

    pub fn find(world: &World, entity: Entity) -> Self {
        world.get::<&Data>(entity).map_or_else(
            |_| Data::Name("???".to_string()),
            |data| data.deref().clone(),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct NounData {
    singular: String,
    plural: String,
}

impl NounData {
    pub fn new(singular: &str, plural: &str) -> Self {
        NounData {
            singular: singular.to_string(),
            plural: plural.to_string(),
        }
    }

    pub fn singular(&self) -> &str {
        &self.singular
    }

    pub fn with_adjective(&self, adjective: &str) -> NounData {
        NounData {
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

pub fn as_grouped_text_list(data: Vec<Data>) -> String {
    let mut names = Vec::new();
    let mut nouns = HashMap::new();

    for name_data in data {
        match name_data {
            Data::Name(name) => names.push(name),
            Data::Noun(noun) => *nouns.entry(noun).or_insert(0) += 1,
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
        .join(", ")
}
