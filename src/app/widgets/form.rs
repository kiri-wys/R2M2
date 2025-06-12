pub mod input_box;

use std::{borrow::Cow, marker::PhantomData, str::FromStr};

use crossterm::event::{Event, KeyCode};
use input_box::InputBox;
use rand::Rng as _;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Position, Rect},
    style::Color,
    widgets::{Clear, StatefulWidget, Widget},
};
use tracing::error;
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::mods::Tag;

pub trait FormSpec {
    const PROMPTS: &'static [&'static str];
    type Output;

    fn try_into_key(&mut self, key: usize, content: &str) -> FormPoll<Self::Output>;
    fn read_key(&self, key: usize) -> Option<String>;
    fn selected_color(&self) -> Color;
}
#[derive(Default)]
pub struct Form<T: FormSpec> {
    pub widget: FormWidget<T>,
    pub state: FormState<T>,
}
impl<T: FormSpec> Form<T> {
    pub fn render_widget(&mut self, frame: &mut Frame, area: Rect) {
        let &mut Form {
            ref widget,
            ref mut state,
        } = self;
        frame.render_stateful_widget(widget, area, state);
        frame.set_cursor_position(state.cursor_pos);
    }
}

#[derive(Default)]
pub struct FormWidget<T> {
    phantom: PhantomData<T>,
}
#[derive(Default)]
pub struct FormState<T: FormSpec> {
    pub background_color: Color,
    selected_color: Color,
    curr_buff: Input,
    index: usize,
    spec: T,
    cursor_pos: Position,
}
#[derive(PartialEq, Eq)]
pub enum FormPoll<T> {
    NextField,
    Invalid,
    Done(T),
}
impl<T: FormSpec> StatefulWidget for &FormWidget<T> {
    type State = FormState<T>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let prompts = T::PROMPTS;
        let layout = Layout::vertical(prompts.iter().map(|_| Constraint::Length(3))).split(area);

        Clear.render(area, buf);
        let selected_layout = layout[state.index];
        let width = selected_layout.width.saturating_sub(4);
        let scroll = state.curr_buff.visual_scroll(width.into());
        let cur = state.curr_buff.visual_cursor().saturating_sub(scroll) as u16;

        state.cursor_pos.x = area.x + 1 + cur;
        state.cursor_pos.y = area.y + 1 + (3 * state.index as u16);
        for (idx, prompt) in prompts.iter().enumerate() {
            let selected = idx == state.index;
            let text = if selected {
                Cow::Borrowed(state.curr_buff.value())
            } else {
                match state.spec.read_key(idx) {
                    Some(v) => Cow::Owned(v),
                    None => "".into(),
                }
            };
            let foreground_color = if selected {
                state.spec.selected_color()
            } else {
                Color::default()
            };
            let input = InputBox {
                title: prompt,
                buffer: text.as_ref(),
                background_color: state.background_color,
                foreground_color,
                selected,
                scroll: scroll as u16,
            };
            input.render(layout[idx], buf);
        }
    }
}
impl<T: FormSpec> FormState<T> {
    #[must_use]
    pub fn handle_input(&mut self, ev: &Event) -> Option<T::Output> {
        match ev {
            Event::Key(key) if key.is_press() => match key.code {
                KeyCode::Insert | KeyCode::Tab | KeyCode::Enter => {
                    if let FormPoll::Done(v) = self.next() {
                        return Some(v);
                    }
                }
                _ => {
                    self.curr_buff.handle_event(ev);
                    self.spec.try_into_key(self.index, self.curr_buff.value());
                }
            },
            e => {
                self.curr_buff.handle_event(e);
                self.spec.try_into_key(self.index, self.curr_buff.value());
            }
        }
        self.selected_color = self.spec.selected_color();
        None
    }
    pub fn next(&mut self) -> FormPoll<T::Output> {
        let res = self.spec.try_into_key(self.index, self.curr_buff.value());
        if let FormPoll::NextField = res {
            self.index += 1;
            match self.spec.read_key(self.index) {
                Some(next) => {
                    // PERF: The clone can't be avoided unless a
                    // native type is rolled out.
                    self.curr_buff = self.curr_buff.clone().with_value(next);
                }
                None => {
                    self.curr_buff.value_and_reset();
                }
            }
        }
        res
    }
}
impl<T: FormSpec + Default> FormState<T> {
    pub fn reset(&mut self) {
        let FormState {
            background_color: _,
            selected_color,
            curr_buff,
            index,
            spec,
            cursor_pos,
        } = self;
        curr_buff.value_and_reset();
        *index = 0;
        *spec = T::default();
        *selected_color = spec.selected_color();
        *cursor_pos = Default::default();
    }
}
fn random_color() -> (u8, u8, u8) {
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

#[derive(Debug)]
pub struct TagForm {
    name: String,
    score: Option<u64>,
    color: Color,
}

impl FormSpec for TagForm {
    const PROMPTS: &'static [&'static str] = &["Name", "Score", "Color"];
    type Output = Tag;

    fn try_into_key(&mut self, key: usize, content: &str) -> FormPoll<Self::Output> {
        match key {
            0 => {
                if !content.trim().is_empty() {
                    self.name = content.trim().to_owned();
                    FormPoll::NextField
                } else {
                    FormPoll::Invalid
                }
            }
            1 => match content.trim().parse() {
                Ok(n) => {
                    self.score = Some(n);
                    FormPoll::NextField
                }
                Err(_) => FormPoll::Invalid,
            },
            2 => match Color::from_str(content) {
                Ok(c) => {
                    self.color = c;
                    FormPoll::Done(Tag {
                        name: self.name.clone(),
                        score: self.score.unwrap(),
                        color: self.color,
                    })
                }
                Err(_) => FormPoll::Invalid,
            },
            _ => FormPoll::Done(Tag {
                name: self.name.clone(),
                score: self.score.unwrap(),
                color: self.color,
            }),
        }
    }

    fn read_key(&self, key: usize) -> Option<String> {
        match key {
            0 => {
                if !self.name.is_empty() {
                    Some(self.name.clone())
                } else {
                    None
                }
            }
            1 => self.score.map(|v| v.to_string()),
            2 => Some(self.color.to_string()),
            _ => {
                error!("Invalid read with index {key} for TagForm");
                None
            }
        }
    }

    fn selected_color(&self) -> Color {
        self.color
    }
}
impl Default for TagForm {
    fn default() -> Self {
        let (r, g, b) = random_color();
        Self {
            color: Color::Rgb(r, g, b),
            name: Default::default(),
            score: Default::default(),
        }
    }
}
