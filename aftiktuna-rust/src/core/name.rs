use hecs::{Entity, EntityRef, World};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::Deref;

use super::CreatureAttribute;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum NameData {
    Name(String),
    Noun(Option<Adjective>, Noun),
}

impl NameData {
    pub fn base(&self) -> String {
        match self {
            NameData::Name(name) => name.to_owned(),
            NameData::Noun(adjective, noun) => format!(
                "{adjective}{entity}",
                adjective = format_option_with_space(adjective.as_ref()),
                entity = noun.singular
            ),
        }
    }
    pub fn plural(&self) -> String {
        match self {
            NameData::Name(name) => name.to_owned(),
            NameData::Noun(adjective, noun) => format!(
                "{adjective}{entity}",
                adjective = format_option_with_space(adjective.as_ref()),
                entity = noun.plural
            ),
        }
    }
    pub fn definite(&self) -> String {
        match self {
            NameData::Name(name) => name.to_string(),
            NameData::Noun(adjective, noun) => format!(
                "the {adjective}{entity}",
                adjective = format_option_with_space(adjective.as_ref()),
                entity = noun.singular
            ),
        }
    }

    pub fn find_option_by_ref(entity_ref: EntityRef) -> Option<Self> {
        if let Some(name) = entity_ref.get::<&Name>()
            && name.is_known
        {
            Some(Self::Name(name.name.clone()))
        } else {
            entity_ref.get::<&Noun>().map(|noun| {
                Self::Noun(
                    entity_ref.get::<&Adjective>().as_deref().cloned(),
                    noun.deref().clone(),
                )
            })
        }
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
        match &self.0 {
            NameData::Name(name) => name.to_owned(),
            NameData::Noun(adjective, noun) => {
                format!(
                    "{adjective}{attribute}{entity}",
                    adjective = format_option_with_space(adjective.as_ref()),
                    attribute = format_option_with_space(self.1),
                    entity = noun.singular
                )
            }
        }
    }

    pub fn definite(&self) -> String {
        match (&self.0, self.1) {
            (NameData::Name(name), _) => name.to_owned(),
            (NameData::Noun(adjective, noun), attribute) => {
                format!(
                    "the {adjective}{attribute}{entity}",
                    adjective = format_option_with_space(adjective.as_ref()),
                    attribute = format_option_with_space(attribute),
                    entity = noun.singular
                )
            }
        }
    }
}

fn format_option_with_space(option: Option<impl Display>) -> String {
    option.map_or(String::default(), |value| format!("{value} "))
}

impl Default for NameData {
    fn default() -> Self {
        Self::Name("???".to_owned())
    }
}

impl<'a> From<NameQuery<'a>> for NameData {
    fn from(query: NameQuery<'a>) -> Self {
        let (name, noun, adjective) = query;
        if let Some(name) = name
            && name.is_known
        {
            Self::Name(name.name.clone())
        } else {
            noun.map(|noun| Self::Noun(adjective.cloned(), noun.clone()))
                .unwrap_or_default()
        }
    }
}

pub type NameQuery<'a> = (Option<&'a Name>, Option<&'a Noun>, Option<&'a Adjective>);

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    #[serde(default)]
    article: IndefiniteArticle,
}

impl Noun {
    pub fn new(singular: &str, plural: &str, article: IndefiniteArticle) -> Self {
        Self {
            singular: singular.to_string(),
            plural: plural.to_string(),
            article,
        }
    }

    pub fn singular(&self) -> &str {
        &self.singular
    }

    pub fn plural(&self) -> &str {
        &self.plural
    }

    pub fn for_count(&self, count: u16) -> &str {
        if count == 1 {
            &self.singular
        } else {
            &self.plural
        }
    }

    pub fn with_text_count(&self, count: u16, article: ArticleKind) -> String {
        self.with_count(None, count, article, CountFormat::Text)
    }

    pub fn with_count(
        &self,
        adjective: Option<&Adjective>,
        count: u16,
        article: ArticleKind,
        format: CountFormat,
    ) -> String {
        if article == ArticleKind::The {
            if count == 1 {
                format!(
                    "the {adjective}{name}",
                    adjective = format_option_with_space(adjective),
                    name = self.singular(),
                )
            } else {
                format!(
                    "the {count} {adjective}{name}",
                    count = format.apply(count),
                    adjective = format_option_with_space(adjective),
                    name = self.for_count(count),
                )
            }
        } else if article == ArticleKind::A && count == 1 {
            format!(
                "{a} {adjective}{name}",
                a = self.article,
                adjective = format_option_with_space(adjective),
                name = self.singular(),
            )
        } else {
            format!(
                "{count} {adjective}{name}",
                count = format.apply(count),
                adjective = format_option_with_space(adjective),
                name = self.for_count(count),
            )
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Adjective(pub String);

impl Display for Adjective {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
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
pub enum ArticleKind {
    The,
    A,
    One,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IndefiniteArticle {
    #[default]
    A,
    An,
}

impl Display for IndefiniteArticle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            IndefiniteArticle::A => "a",
            IndefiniteArticle::An => "an",
        })
    }
}

pub fn names_with_counts(
    data: impl IntoIterator<Item = NameData>,
    article: ArticleKind,
    format: CountFormat,
) -> Vec<String> {
    let mut names = Vec::new();
    let mut nouns = HashMap::new();

    for name_data in data {
        match name_data {
            NameData::Name(name) => names.push(name),
            NameData::Noun(adjective, noun) => *nouns.entry((adjective, noun)).or_insert(0) += 1,
        }
    }

    names
        .into_iter()
        .chain(nouns.into_iter().map(|((adjective, noun), count)| {
            noun.with_count(adjective.as_ref(), count, article, format)
        }))
        .collect::<Vec<String>>()
}
