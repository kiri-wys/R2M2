//! Models related to Rimworld
//!
//! [`Mod`] contains all the relevant data to a Rimworld mod.
//! [`OrderedVec<T>`] is a container of <T> that is internally ordered
//! [`TagSpans`] may be used to get a display representation to
//! a collection of [`Tag`]. It blends background_color with each [`Tag`]'s color.
pub mod game;

use std::{cmp::Ordering, hash::Hash};

use game::ModMetaData;

use ratatui::{
    style::{Color, Style, Stylize},
    text::{Line, Span},
};
use serde::{Deserialize, Serialize};

pub trait Item {
    fn identifier(&self) -> &str;
    fn patch(&mut self, other: Self);
    fn vec_order(&self, other: &Self) -> Ordering;
}
#[derive(Default, Clone, Debug, Serialize)]
pub struct OrderedVec<T: Item> {
    data: Vec<T>,
}
impl<'de, T> Deserialize<'de> for OrderedVec<T>
where
    T: Item + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct UnorderedVec<T> {
            data: Vec<T>,
        }
        let res: UnorderedVec<T> = UnorderedVec::deserialize(deserializer)?;
        Ok(res.data.into())
    }
}
impl<T: Item> OrderedVec<T> {
    pub fn get_by_name(&self, name: &str) -> Option<&T> {
        self.data.iter().find(|&item| item.identifier() == name)
    }
    fn get_mut_by_name(&mut self, name: &str) -> Option<&mut T> {
        self.data.iter_mut().find(|item| item.identifier() == name)
    }

    pub fn upsert(&mut self, other: T) {
        if let Some(existing) = self.get_mut_by_name(other.identifier()) {
            existing.patch(other);
            return;
        }
        let idx = self
            .data
            .binary_search_by(|curr| curr.vec_order(&other))
            .unwrap_or_else(|i| i);
        self.data.insert(idx, other);
    }
    pub const fn len(&self) -> usize {
        self.data.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn get<I>(&self, index: I) -> Option<&I::Output>
    where
        I: std::slice::SliceIndex<[T]>,
    {
        self.data.get(index)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.data.iter()
    }
}

impl<T> From<Vec<T>> for OrderedVec<T>
where
    T: Item,
{
    fn from(mut value: Vec<T>) -> Self {
        value.sort_by(|a, b| a.vec_order(b));
        Self { data: value }
    }
}

impl OrderedVec<Mod> {
    pub fn upsert_tag_to(&mut self, mod_idx: usize, tag: Tag) {
        self.data.get_mut(mod_idx).unwrap().tags.upsert(tag);
        self.data.sort_by(|a, b| a.vec_order(b));
    }
}
impl OrderedVec<Tag> {
    pub fn spans(&self, bg_color: Color, selected_tag: SelectedTag) -> TagSpans {
        TagSpans {
            idx: 0,
            selected_tag,
            bg_color,
            tags: self,
        }
    }
    pub fn styled_line(&self, bg_color: Color, is_selected: bool) -> Line {
        // this works but looks weird, investigate with real data
        //for tag in self.0.as_slice().windows(2) {
        //let (l, r) = (&tag[0], &tag[1]);
        //let bg = blend_color(l.color, bg_color, 0.8);
        //let (text_color, text_bg) = if !is_selected {
        //(l.color, bg)
        //} else {
        //(bg, l.color)
        //};
        //buff.push(Span::styled(
        //format!(" {}", l.name),
        //Style::default().bold().fg(text_color).bg(text_bg),
        //));
        //let bg = if is_selected {
        //r.color
        //} else {
        //blend_color(r.color, bg_color, 0.8)
        //};
        //buff.push(Span::styled(
        //"",
        //Style::default().bold().bg(bg).fg(text_bg),
        //));
        //}
        let mut buff = vec![];
        let selected = if is_selected {
            SelectedTag::All
        } else {
            SelectedTag::None
        };
        for span in TagSpans::new(selected, bg_color, self) {
            buff.push(span);
        }
        Line::from(buff)
    }
}
pub struct TagSpans<'tags> {
    idx: usize,
    selected_tag: SelectedTag,
    bg_color: Color,
    tags: &'tags OrderedVec<Tag>,
}
pub enum SelectedTag {
    All,
    Index(usize),
    None,
}

impl<'tags> TagSpans<'tags> {
    fn new(selected_tag: SelectedTag, bg_color: Color, tags: &'tags OrderedVec<Tag>) -> Self {
        Self {
            idx: 0,
            selected_tag,
            bg_color,
            tags,
        }
    }
}
impl<'spans> Iterator for TagSpans<'spans> {
    type Item = Span<'static>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.tags.data.is_empty() {
            return if self.idx == 1 {
                None
            } else {
                self.idx = 1;
                Some(Span::styled(
                    "N/A",
                    Style::default().italic().fg(Color::Rgb(0x8d, 0x8d, 0x8d)),
                ))
            };
        }

        let tag = self.tags.data.get(self.idx)?;
        let bg = Tag::blend_color(tag.color, self.bg_color, 0.8);
        let (text_color, text_bg) = match self.selected_tag {
            SelectedTag::All => (bg, tag.color),
            SelectedTag::Index(i) => {
                if i == self.idx {
                    (bg, tag.color)
                } else {
                    (tag.color, bg)
                }
            }
            SelectedTag::None => (tag.color, bg),
        };
        self.idx += 1;
        Some(Span::styled(
            format!(" {} ", tag.name),
            Style::default().bold().fg(text_color).bg(text_bg),
        ))
    }
}

impl Item for Mod {
    fn identifier(&self) -> &str {
        &self.metadata.package_id
    }

    fn patch(&mut self, other: Self) {
        *self = other;
    }

    fn vec_order(&self, other: &Self) -> Ordering {
        match (self.tags.data.first(), other.tags.data.first()) {
            (None, None) => self.metadata.name.cmp(&other.metadata.name),
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(_), Some(_)) => match self
                .tags
                .data
                .iter()
                .map(|v| v.score)
                .cmp(other.tags.data.iter().map(|v| v.score))
            {
                Ordering::Equal => self.metadata.name.cmp(&other.metadata.name),
                other => other,
            },
        }
        /*match self.tags.get(0) {
            Some(st) => {
                match other.tags.get(0) {
                    Some(ot) => {
                        //let a = format!("\n({}: {:#?})", self.metadata.name, st);
                        st.score.cmp(&ot.score)
                    }
                    None => Ordering::Less,
                }
            }
            None => match other.tags.get(0) {
                Some(_) => Ordering::Greater,
                None => self.metadata.name.cmp(&other.metadata.name),
            },
        }*/
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Mod {
    pub metadata: ModMetaData,
    tags: OrderedVec<Tag>,
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
impl Item for Tag {
    fn identifier(&self) -> &str {
        &self.name
    }

    fn patch(&mut self, other: Self) {
        let Tag {
            name: _,
            score,
            color,
        } = other;
        self.score = score;
        self.color = color;
    }

    fn vec_order(&self, other: &Self) -> Ordering {
        self.score
            .cmp(&other.score)
            .then(self.name.cmp(&other.name))
    }
}
#[derive(Default, Clone, Debug, Deserialize, Serialize)]
pub struct Tag {
    pub name: String,
    pub score: u64,
    pub color: Color,
}
impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Eq for Tag {}
impl Hash for Tag {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}
impl Tag {
    fn blend_color(color: Color, blend: Color, factor: f32) -> Color {
        if let (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) = (color, blend) {
            let new_r = r1 as f32 + (r2 as f32 - r1 as f32) * factor;
            let new_g = g1 as f32 + (g2 as f32 - g1 as f32) * factor;
            let new_b = b1 as f32 + (b2 as f32 - b1 as f32) * factor;
            Color::Rgb(new_r as u8, new_g as u8, new_b as u8)
        } else {
            color
        }
    }
}
