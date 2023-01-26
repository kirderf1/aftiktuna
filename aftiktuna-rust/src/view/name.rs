use crate::view::DisplayInfo;
use hecs::{Entity, World};
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Data {
    Name(String),
    Noun(String),
}

impl Data {
    pub fn from_name(name: &str) -> Self {
        Data::Name(name.to_string())
    }
    pub fn from_noun(noun: &str) -> Self {
        Data::Noun(noun.to_string())
    }

    pub fn base(&self) -> &str {
        match self {
            Data::Name(name) | Data::Noun(name) => name,
        }
    }
    pub fn definite(&self) -> String {
        match self {
            Data::Name(name) => name.to_string(),
            Data::Noun(name) => format!("the {}", name),
        }
    }

    pub fn matches(&self, string: &str) -> bool {
        self.base().eq_ignore_ascii_case(string)
    }

    pub fn find(world: &World, entity: Entity) -> Self {
        world.get::<&DisplayInfo>(entity).map_or_else(
            |_| Data::Name("???".to_string()),
            |info| info.name_data.clone(),
        )
    }
}

pub fn group_data(data: Vec<Data>) -> (Vec<String>, Vec<(String, i32)>) {
    let mut names = Vec::new();
    let mut nouns = HashMap::new();

    for name_data in data {
        match name_data {
            Data::Name(name) => names.push(name),
            Data::Noun(noun) => *nouns.entry(noun).or_insert(0) += 1,
        }
    }

    (names, nouns.into_iter().collect())
}
