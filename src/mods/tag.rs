use std::cmp::Ordering;
use std::hash::Hash;

use ratatui::{
    style::{Color, Style, Stylize as _},
    text::{Line, Span},
};
use serde::{Deserialize, Serialize};

use super::{Item, OrderedItems};

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

impl OrderedItems<Tag> {
    pub fn spans(&self, bg_color: Color, selected_tag: SelectedTag) -> TagSpans {
        TagSpans {
            idx: 0,
            selected_tag,
            bg_color,
            tags: self,
        }
    }
    pub fn styled_line(&self, bg_color: Color, is_selected: bool) -> Line {
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

pub enum SelectedTag {
    All,
    Index(usize),
    None,
}
pub struct TagSpans<'tags> {
    idx: usize,
    selected_tag: SelectedTag,
    bg_color: Color,
    tags: &'tags OrderedItems<Tag>,
}

impl<'tags> TagSpans<'tags> {
    fn new(selected_tag: SelectedTag, bg_color: Color, tags: &'tags OrderedItems<Tag>) -> Self {
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
        if self.tags.is_empty() {
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

        let tag = self.tags.get(self.idx)?;
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
