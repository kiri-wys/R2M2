//! Models related to Rimworld
//!
//! [`Mod`] contains all the relevant data to a Rimworld mod.
//! [`Tags`] is a container of [`Tag`] that is internally ordered
//! by its score and name in that order.
//! [`TagSpans`] may be used to get a display representation to
//! [`Tags`]. It blends background_color with each [`Tag`]'s color.
pub mod game;

use std::hash::Hash;

use game::ModMetaData;

use ratatui::{
    style::{Color, Style, Stylize},
    text::{Line, Span},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Mod {
    pub metadata: ModMetaData,
    pub tags: Tags,
}

#[derive(Default, Clone, Debug, Deserialize, Serialize)]
pub struct Tags(Vec<Tag>);
impl<Iter> From<Iter> for Tags
where
    Iter: Iterator<Item = String>,
{
    fn from(value: Iter) -> Self {
        let (mins, maxs) = value.size_hint();
        let s = maxs.unwrap_or(mins);
        let mut buff = Vec::with_capacity(s);
        let mut seen = std::collections::HashSet::new();
        for (idx, name) in value.enumerate() {
            if !seen.insert(name.clone()) {
                continue;
            }
            buff.push(Tag {
                name,
                color: Color::White,
                score: idx as u64,
            });
        }
        buff.sort_by(|a, b| a.score.cmp(&b.score).then_with(|| a.name.cmp(&b.name)));
        Tags(buff)
    }
}
impl Tags {
    pub fn spans(&self, bg_color: Color, selected_idx: Option<usize>) -> TagSpans {
        TagSpans {
            idx: 0,
            selected_idx,
            bg_color,
            tags: self,
        }
    }
    pub fn styled_line(&self, bg_color: Color, selected_idx: Option<usize>) -> Line {
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
        for span in TagSpans::new(selected_idx, bg_color, self) {
            buff.push(span);
        }
        Line::from(buff)
    }

    pub fn get_by_name(&self, name: &str) -> Option<&Tag> {
        self.0.iter().find(|&tag| tag.name == name)
    }
    pub fn get_mut_by_name(&mut self, name: &str) -> Option<&mut Tag> {
        self.0.iter_mut().find(|tag| tag.name == name)
    }

    pub fn upsert(&mut self, tag: Tag) {
        if let Some(existing) = self.get_mut_by_name(&tag.name) {
            let Tag {
                name: _,
                score,
                color,
            } = tag;
            existing.score = score;
            existing.color = color;
            return;
        }
        let idx = self
            .0
            .binary_search_by(|curr| {
                curr.score
                    .cmp(&tag.score)
                    .then_with(|| curr.name.cmp(&tag.name))
            })
            .unwrap_or_else(|i| i);
        self.0.insert(idx, tag);
    }
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get<I>(&self, index: I) -> Option<&I::Output>
    where
        I: std::slice::SliceIndex<[Tag]>,
    {
        self.0.get(index)
    }
}
pub struct TagSpans<'tags> {
    idx: usize,
    selected_idx: Option<usize>,
    bg_color: Color,
    tags: &'tags Tags,
}

impl<'tags> TagSpans<'tags> {
    fn new(selected_idx: Option<usize>, bg_color: Color, tags: &'tags Tags) -> Self {
        Self {
            idx: 0,
            selected_idx,
            bg_color,
            tags,
        }
    }
}
impl<'spans> Iterator for TagSpans<'spans> {
    type Item = Span<'static>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.tags.0.is_empty() {
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

        let tag = self.tags.0.get(self.idx)?;
        let bg = Tag::blend_color(tag.color, self.bg_color, 0.8);
        let (text_color, text_bg) = if self.selected_idx.is_none_or(|i| i != self.idx) {
            (tag.color, bg)
        } else {
            (bg, tag.color)
        };
        self.idx += 1;
        Some(Span::styled(
            format!(" {} ", tag.name),
            Style::default().bold().fg(text_color).bg(text_bg),
        ))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
