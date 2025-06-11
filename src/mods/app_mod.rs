use std::cmp::Ordering;

use ratatui::{style::Color, text::Line};
use serde::{Deserialize, Serialize};

use super::{Item, OrderedItems, game::ModMetaData, tag::Tag};

impl Item for Mod {
    fn identifier(&self) -> &str {
        &self.metadata.package_id
    }

    fn patch(&mut self, other: Self) {
        *self = other;
    }

    fn vec_order(&self, other: &Self) -> Ordering {
        match (self.tags.first(), other.tags.first()) {
            (None, None) => self.metadata.name.cmp(&other.metadata.name),
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(_), Some(_)) => match self
                .tags
                .iter()
                .map(|v| v.score)
                .cmp(other.tags.iter().map(|v| v.score))
            {
                Ordering::Equal => self.metadata.name.cmp(&other.metadata.name),
                other => other,
            },
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Mod {
    pub metadata: ModMetaData,
    tags: OrderedItems<Tag>,
}
impl Mod {
    pub fn new(metadata: ModMetaData) -> Self {
        Self {
            metadata,
            tags: Default::default(),
        }
    }

    pub fn tags_styled_line(&self, bg_color: Color, is_selected: bool) -> Line {
        self.tags.styled_line(bg_color, is_selected)
    }
}

impl OrderedItems<Mod> {
    pub fn upsert_tag_to(&mut self, mod_idx: usize, tag: Tag) {
        self.get_mut(mod_idx).unwrap().tags.upsert(tag);
        self.sort();
    }
}
