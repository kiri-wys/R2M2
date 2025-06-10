use std::hash::Hash;

use rand::Rng;
use ratatui::{
    style::{Color, Modifier, Style, Stylize},
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
        for (idx, name) in value.enumerate() {
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

    // TODO: if it exist patch it, otherwise append it
    pub fn insert(&mut self, tag: Tag) {
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
        let bg = blend_color(tag.color, self.bg_color, 0.8);
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
pub(crate) fn random_color() -> (u8, u8, u8) {
    // Adapted from https://docs.rs/hsv
    fn is_between(value: f64, min: f64, max: f64) -> bool {
        min <= value && value < max
    }

    let mut rng = rand::rng();
    let h: f64 = rng.random_range(0.0..360.0);
    let s = rng.random_range(0.7..1.0);
    let v = rng.random_range(0.7..1.0);
    let c = v * s;

    let h = h / 60.0;

    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());

    let m = v - c;

    let (r, g, b): (f64, f64, f64) = if is_between(h, 0.0, 1.0) {
        (c, x, 0.0)
    } else if is_between(h, 1.0, 2.0) {
        (x, c, 0.0)
    } else if is_between(h, 2.0, 3.0) {
        (0.0, c, x)
    } else if is_between(h, 3.0, 4.0) {
        (0.0, x, c)
    } else if is_between(h, 4.0, 5.0) {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModMetaData {
    pub name: String,
    #[serde(
        default,
        serialize_with = "wrap_strings",
        deserialize_with = "unwrap_strings"
    )]
    pub supported_versions: Vec<String>,
    #[serde(default)]
    pub mod_dependencies_by_version: ModDependencies,
    #[serde(
        default,
        serialize_with = "wrap_strings",
        deserialize_with = "unwrap_strings"
    )]
    pub load_after: Vec<String>,
    pub description: String,
    pub package_id: String,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ModDependencies {
    #[serde(
        default,
        rename = "v1.0",
        serialize_with = "wrap_deps",
        deserialize_with = "unwrap_deps"
    )]
    pub v1_0: Vec<Dependency>,
    #[serde(
        default,
        rename = "v1.1",
        serialize_with = "wrap_deps",
        deserialize_with = "unwrap_deps"
    )]
    pub v1_1: Vec<Dependency>,
    #[serde(
        default,
        rename = "v1.2",
        serialize_with = "wrap_deps",
        deserialize_with = "unwrap_deps"
    )]
    pub v1_2: Vec<Dependency>,
    #[serde(
        default,
        rename = "v1.3",
        serialize_with = "wrap_deps",
        deserialize_with = "unwrap_deps"
    )]
    pub v1_3: Vec<Dependency>,
    #[serde(
        default,
        rename = "v1.4",
        serialize_with = "wrap_deps",
        deserialize_with = "unwrap_deps"
    )]
    pub v1_4: Vec<Dependency>,
    #[serde(
        default,
        rename = "v1.5",
        serialize_with = "wrap_deps",
        deserialize_with = "unwrap_deps"
    )]
    pub v1_5: Vec<Dependency>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Dependency {
    pub package_id: String,
    pub display_name: String,
}

fn wrap_deps<S>(deps: &[Dependency], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    wrap_list(deps, serializer)
}
fn wrap_strings<S>(deps: &[String], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    wrap_list(deps, serializer)
}
fn unwrap_deps<'de, D>(deserializer: D) -> Result<Vec<Dependency>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    unwrap_list(deserializer)
}
fn unwrap_strings<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    unwrap_list(deserializer)
}
#[inline]
fn wrap_list<S, R>(vec: &[R], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    R: Serialize,
{
    #[derive(Serialize)]
    struct List<'a, R> {
        li: &'a [R],
    }

    List { li: vec }.serialize(serializer)
}
#[inline]
fn unwrap_list<'de, D, R>(deserializer: D) -> Result<Vec<R>, D::Error>
where
    D: serde::Deserializer<'de>,
    R: Default + serde::Deserialize<'de>,
{
    #[derive(Deserialize)]
    struct List<R> {
        #[serde(default)]
        li: Vec<R>,
    }
    Ok(List::deserialize(deserializer)?.li)
}
