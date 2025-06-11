use std::{collections::HashMap, str::FromStr};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use rand::Rng as _;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Cell, List, ListState, Paragraph, Row, Table, TableState},
};
use serde::{Deserialize, Serialize};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::mods::{Mod, OrderedVec, Tag};

#[derive(Default)]
pub enum Mode {
    #[default]
    Normal,
    CreateTag,
    ShowTags,
    Insert,
}
impl Mode {
    pub fn str_repr(&self) -> &'static str {
        match self {
            Mode::Normal => " NORMAL ",
            Mode::CreateTag => " CREATE TAG ",
            Mode::ShowTags => " LISTING TAG ",
            Mode::Insert => " INSERT ",
        }
    }

    fn color_repr(&self) -> Color {
        match self {
            Mode::Normal => Color::Rgb(0x45, 0x89, 0xff),
            Mode::CreateTag => Color::Rgb(0x42, 0xbe, 0x65),
            Mode::ShowTags => Color::Rgb(0xfe, 0x83, 0x2b),
            Mode::Insert => Color::Rgb(0x42, 0xbe, 0x65),
        }
    }
}
#[derive(Default)]
pub struct Model {
    should_close: bool,
    current_mode: Mode,
    movement_delta: String,
    input_buffer: Input,
    input_collection: HashMap<String, String>,
    input_index: usize,
    input_special: Color,
    table_state: TableState,
    list_state: ListState,
    // keeping queries cached introduces a whole set of problems and
    // it might not even be worth, TODO: benchmark
    //mods_view: Vec<Mod>,
    persistent: Persistent,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Persistent {
    pub mods: OrderedVec<Mod>,
    pub tags: OrderedVec<Tag>,
}
impl Default for Persistent {
    fn default() -> Self {
        Self {
            mods: vec![].into(),
            tags: Default::default(),
        }
    }
}

impl Model {
    pub fn should_close(&self) -> bool {
        self.should_close
    }
    pub fn set_persistent(&mut self, p: Persistent) {
        if !p.mods.is_empty() {
            self.table_state.select_first();
        }
        if !p.mods.is_empty() {
            self.list_state.select_first();
        }
        self.persistent = p;
    }
    pub fn result(self) -> Persistent {
        self.persistent
    }

    pub fn view(&mut self, f: &mut Frame) {
        let area = f.area();
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(1), Constraint::Max(1)])
            .split(area);

        let table_color = Color::Rgb(0x16, 0x16, 0x16);
        let mut rows = vec![];
        // TODO: Keep in sync when persistent mods is updated
        for (idx, game_mod) in self.persistent.mods.iter().enumerate() {
            let line_num = match self.table_state.selected() {
                Some(s) => {
                    if s == idx {
                        format!("{:<3}", idx)
                    } else {
                        let n = if idx < s { s - idx } else { idx - s };
                        format!("{:>3}", n)
                    }
                }
                None => format!("{:>3}", idx),
            };
            let selected = self.table_state.selected().is_some_and(|v| v == idx);
            let mut name = Cell::from(game_mod.metadata.name.to_owned());
            if selected {
                name = name.bg(Color::Rgb(0x39, 0x39, 0x39));
            }
            rows.push(Row::new(vec![
                Cell::from(line_num),
                name,
                Cell::from(game_mod.tags_styled_line(table_color, selected)),
            ]));
        }

        let widths = [
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Percentage(60),
        ];
        let table = Table::new(rows, widths)
            .block(Block::new().title("Table"))
            .row_highlight_style(Style::new().bold())
            .highlight_symbol(">>")
            .bg(table_color);

        f.render_stateful_widget(table, main_layout[0], &mut self.table_state);
        self.draw_line(f, main_layout[1]);
        self.draw_popup(f, area);
    }

    #[inline]
    fn draw_line(&self, f: &mut Frame, rect: Rect) {
        let bar_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Min(1), Constraint::Min(1)])
            .split(rect);

        let mode_color = self.current_mode.color_repr();
        let bar = Color::Rgb(0x0b, 0x0b, 0x0b);
        let hint = if !self.movement_delta.is_empty() {
            self.movement_delta.to_string()
        } else {
            match self.current_mode {
                Mode::Normal => "Press ? for help, 'q' to quit",
                Mode::CreateTag | Mode::ShowTags | Mode::Insert => "Press ESC to go back",
            }
            .to_string()
        };
        f.render_widget(
            Line::from(vec![
                Span::styled(
                    self.current_mode.str_repr(),
                    Style::default().bg(mode_color).fg(bar).bold(),
                ),
                Span::styled("", Style::default().fg(mode_color).bg(bar)),
            ])
            .bg(bar),
            bar_layout[0],
        );
        f.render_widget(
            Line::from(vec![Span::styled(
                hint,
                Style::default().fg(mode_color).bg(bar),
            )])
            .alignment(Alignment::Right)
            .bg(bar),
            bar_layout[1],
        );
    }

    #[inline]
    fn draw_popup(&mut self, f: &mut Frame, area: Rect) {
        match self.current_mode {
            // TODO: This is horrid, fix
            Mode::CreateTag => {
                let area =
                    Self::popup_area(area, Constraint::Percentage(30), Constraint::Max(3 * 3));
                let sections =
                    Layout::vertical([Constraint::Max(3), Constraint::Max(3), Constraint::Max(3)])
                        .split(area);

                f.render_widget(ratatui::widgets::Clear, area);
                for (idx, (s, n)) in sections.iter().zip(["Name", "Score", "Color"]).enumerate() {
                    let fallback = self
                        .input_collection
                        .get(n)
                        .map(|s| s.to_owned())
                        .unwrap_or_default();

                    Self::input_box(
                        f,
                        *s,
                        n,
                        if idx == self.input_index {
                            if let Some(ss) = self.input_collection.remove(n) {
                                self.input_buffer =
                                    self.input_buffer.clone().with_value(ss.to_string());
                            }
                            Some(&self.input_buffer)
                        } else {
                            None
                        },
                        &fallback,
                        if idx == 0 || idx == 2 {
                            Some(self.input_special)
                        } else {
                            None
                        },
                    );
                }
            }
            Mode::ShowTags => {
                let area =
                    Self::popup_area(area, Constraint::Percentage(70), Constraint::Percentage(60));
                let bg_color = Color::Rgb(0x26, 0x26, 0x26);
                let p = Paragraph::new(vec![self.persistent.tags.styled_line(bg_color, false)])
                    .block(Block::bordered().title("Tags"))
                    .bg(bg_color);
                f.render_widget(ratatui::widgets::Clear, area);
                f.render_widget(p, area);
            }
            Mode::Insert => {
                let area =
                    Self::popup_area(area, Constraint::Percentage(70), Constraint::Percentage(60));
                let bg_color = Color::Rgb(0x26, 0x26, 0x26);
                f.render_widget(ratatui::widgets::Clear, area);

                let mut items = vec![];
                let selected = match self.list_state.selected() {
                    Some(i) => crate::mods::SelectedTag::Index(i),
                    None => crate::mods::SelectedTag::None,
                };
                for span in self.persistent.tags.spans(bg_color, selected) {
                    items.push(Line::from(vec![span]));
                }
                let list = List::new(items)
                    .block(Block::bordered().title("Tags"))
                    .bg(bg_color)
                    .highlight_symbol(">>")
                    .repeat_highlight_symbol(true);

                f.render_stateful_widget(list, area, &mut self.list_state);
            }
            _ => {}
        }
    }
    // TODO: This most likely warrants its very own widget
    fn input_box(
        f: &mut Frame,
        section: Rect,
        name: &str,
        input: Option<&Input>,
        fallback: &str,
        color: Option<Color>,
    ) {
        let width = section.width.max(5) - 5;
        let scroll = input
            .map(|i| i.visual_scroll(width as usize))
            .unwrap_or_default();
        let block = Block::bordered().title(name);
        let mut style = Style::default();
        if let Some(c) = color {
            style = style.fg(c);
        }
        let mut name = Paragraph::new(vec![Line::from(vec![Span::styled(
            format!("> {}", input.map(|i| i.value()).unwrap_or(fallback)),
            style,
        )])])
        .scroll((0, scroll as u16))
        .block(block.clone())
        .bg(Color::Rgb(0x26, 0x26, 0x26));

        if input.is_some() {
            name = name.block(block.style(Style::default().fg(Color::Rgb(0xfd, 0xdc, 0x69))));
            let cur = input.unwrap().visual_cursor().max(scroll) - scroll;
            f.set_cursor_position((section.x + cur as u16 + 3, section.y + 1));
        }
        f.render_widget(name, section);
    }
    fn popup_area(area: Rect, constraint_x: Constraint, constraint_y: Constraint) -> Rect {
        let vertical = Layout::vertical([constraint_y]).flex(Flex::Center);
        let horizontal = Layout::horizontal([constraint_x]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }

    pub fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::ClearCommand => self.movement_delta.clear(),
            Message::AppendMovement(ch) => self.movement_delta.push(ch),
            Message::MoveDirection(direction) => {
                let d: usize = self.movement_delta.parse().unwrap_or(1);
                // TODO: This is horrid, refactor to use proper patterns
                match self.current_mode {
                    Mode::Normal => {
                        let new = self.table_state.selected().map(|s| match direction {
                            MoveDirection::Up => s.saturating_sub(d),
                            MoveDirection::Down => s
                                .saturating_add(d)
                                .min(self.persistent.mods.len().saturating_sub(1)),
                            MoveDirection::Left => s.saturating_sub(d),
                            MoveDirection::Right => s
                                .saturating_add(d)
                                .min(self.persistent.mods.len().saturating_sub(1)),
                        });
                        self.table_state.select(new);
                    }
                    Mode::Insert => {
                        let new = self.list_state.selected().map(|s| match direction {
                            MoveDirection::Up => s.saturating_sub(d),
                            MoveDirection::Down => s
                                .saturating_add(d)
                                .min(self.persistent.tags.len().saturating_sub(1)),
                            MoveDirection::Left => s.saturating_sub(d),
                            MoveDirection::Right => s
                                .saturating_add(d)
                                .min(self.persistent.tags.len().saturating_sub(1)),
                        });
                        self.list_state.select(new);
                    }
                    _ => {}
                }
                return Some(Message::ClearCommand);
            }
            Message::PropagateEvent(key) => {
                self.input_buffer.handle_event(&key);
                // TODO: Refactor to make intent clearer
                if let Some(c) = self
                    .input_collection
                    .get("Color")
                    .and_then(|v| Color::from_str(v).ok())
                {
                    self.input_special = c;
                };
            }
            Message::InsertTag => {
                let tag = self
                    .persistent
                    .tags
                    .get(self.list_state.selected().unwrap())
                    .unwrap();
                self.list_state.select_first();
                self.persistent
                    .mods
                    .upsert_tag_to(self.table_state.selected().unwrap(), tag.clone());
                return Some(Message::ChangeMode(Mode::Normal));
            }
            // TODO: This logic shouldn't be responsability of App
            Message::NextField => {
                // TODO: Generalize
                if matches!(self.current_mode, Mode::CreateTag) {
                    match self.input_index {
                        0 => {
                            if !self.input_buffer.value().is_empty() {
                                self.input_collection.insert(
                                    "Name".to_string(),
                                    self.input_buffer.value_and_reset(),
                                );
                                self.input_index += 1;
                            }
                        }
                        1 => {
                            if self.input_buffer.value().parse::<u64>().is_ok() {
                                self.input_collection.insert(
                                    "Score".to_string(),
                                    self.input_buffer.value_and_reset(),
                                );
                                self.input_index += 1;
                            }
                        }
                        _ => {
                            if let Ok(color) = Color::from_str(self.input_buffer.value()) {
                                self.input_collection.insert(
                                    "Color".to_string(),
                                    self.input_buffer.value_and_reset(),
                                );
                                self.persistent.tags.upsert(Tag {
                                    name: self.input_collection.get("Name").unwrap().to_string(),
                                    score: self
                                        .input_collection
                                        .get("Score")
                                        .unwrap()
                                        .parse()
                                        .unwrap(),
                                    color,
                                });
                                self.input_collection.clear();
                                self.input_index = 0;
                                self.list_state.select_first();
                                return Some(Message::ChangeMode(Mode::Normal));
                            }
                        }
                    }
                }
            }
            Message::ChangeMode(mode) => {
                // TODO: Generalize
                if matches!(mode, Mode::CreateTag) {
                    self.input_collection.clear();
                    let (r, g, b) = random_color();
                    self.input_collection.insert(
                        "Color".to_string(),
                        format!("#{:0>2x}{:0>2x}{:0>2x}", r, g, b),
                    );
                    self.input_special = Color::Rgb(r, g, b);
                }
                self.current_mode = mode;
                return Some(Message::ClearCommand);
            }
            Message::Exit => self.should_close = true,
        }
        None
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
pub enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}
pub enum Message {
    ClearCommand,
    AppendMovement(char),
    MoveDirection(MoveDirection),
    PropagateEvent(Event),
    NextField,
    InsertTag,
    ChangeMode(Mode),
    Exit,
}
pub fn try_message(model: &Model, ev: Event) -> Option<Message> {
    match ev {
        Event::Key(key_event) => match key_event.kind {
            KeyEventKind::Press => match model.current_mode {
                Mode::Normal => normal_key_press(key_event),
                Mode::CreateTag => {
                    let res = match key_event.code {
                        KeyCode::Esc => Message::ChangeMode(Mode::Normal),
                        KeyCode::Insert | KeyCode::Tab | KeyCode::Enter => Message::NextField,
                        _ => Message::PropagateEvent(ev),
                    };
                    Some(res)
                }
                Mode::ShowTags => show_tags_key_press(key_event),
                Mode::Insert => insert_key_press(key_event),
            },
            KeyEventKind::Repeat => None,
            KeyEventKind::Release => None,
        },
        Event::Mouse(_) => None,
        _ => None,
    }
}

#[inline]
fn normal_key_press(key: KeyEvent) -> Option<Message> {
    let res = match key.code {
        KeyCode::Esc => Message::ClearCommand,
        KeyCode::Char(c) if c.is_ascii_digit() => Message::AppendMovement(c),
        KeyCode::Char('q') => Message::Exit,
        KeyCode::Char('T') => Message::ChangeMode(Mode::CreateTag),
        KeyCode::Char('t') => Message::ChangeMode(Mode::ShowTags),
        KeyCode::Char('i') => Message::ChangeMode(Mode::Insert),
        KeyCode::Char('?') => todo!(),
        KeyCode::Char('k') | KeyCode::Up => Message::MoveDirection(MoveDirection::Up),
        KeyCode::Char('j') | KeyCode::Down => Message::MoveDirection(MoveDirection::Down),
        KeyCode::Char('h') | KeyCode::Left => Message::MoveDirection(MoveDirection::Left),
        KeyCode::Char('l') | KeyCode::Right => Message::MoveDirection(MoveDirection::Right),
        _ => return None,
    };
    Some(res)
}
#[inline]
fn show_tags_key_press(key: KeyEvent) -> Option<Message> {
    let res = match key.code {
        KeyCode::Esc => Message::ChangeMode(Mode::Normal),
        _ => return None,
    };
    Some(res)
}
#[inline]
fn insert_key_press(key: KeyEvent) -> Option<Message> {
    let res = match key.code {
        KeyCode::Esc | KeyCode::Char('q') => Message::ChangeMode(Mode::Normal),
        KeyCode::Enter => Message::InsertTag,
        KeyCode::Char(c) if c.is_ascii_digit() => Message::AppendMovement(c),
        KeyCode::Char('k') | KeyCode::Up => Message::MoveDirection(MoveDirection::Up),
        KeyCode::Char('j') | KeyCode::Down => Message::MoveDirection(MoveDirection::Down),
        KeyCode::Char('h') | KeyCode::Left => Message::MoveDirection(MoveDirection::Left),
        KeyCode::Char('l') | KeyCode::Right => Message::MoveDirection(MoveDirection::Right),
        _ => return None,
    };
    Some(res)
}
