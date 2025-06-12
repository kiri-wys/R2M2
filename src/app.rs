mod messages;
mod widgets;

pub use messages::try_message;
use widgets::{
    StatusLine,
    form::{Form, TagForm},
};

use messages::{Message, MoveDirection};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Cell, List, ListState, Paragraph, Row, Table, TableState},
};
use serde::{Deserialize, Serialize};

use crate::mods::{OrderedItems, app_mod::Mod, tag::Tag};

#[derive(Clone, Copy, Default)]
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
    table_state: TableState,
    list_state: ListState,
    status_line: StatusLine,
    tag_form: Form<TagForm>,
    // keeping queries cached introduces a whole set of problems and
    // it might not even be worth, TODO: benchmark
    //mods_view: Vec<Mod>,
    persistent: Persistent,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Persistent {
    pub mods: OrderedItems<Mod>,
    pub tags: OrderedItems<Tag>,
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
    pub fn new(persistent: Persistent) -> Self {
        let mut res = Self {
            persistent,
            ..Default::default()
        };

        if !res.persistent.mods.is_empty() {
            res.table_state.select_first();
        }
        if !res.persistent.tags.is_empty() {
            res.list_state.select_first();
        }
        res.status_line.state.background_color = Color::Rgb(0x0b, 0x0b, 0x0b);
        res.status_line.state.change_mode(res.current_mode);
        res.status_line
            .state
            .change_hint(res.current_mode, &res.movement_delta);

        res.tag_form.state.background_color = Color::Rgb(0x26, 0x26, 0x26);
        res
    }
    pub fn should_close(&self) -> bool {
        self.should_close
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
        self.status_line.render_widget(f, main_layout[1]);
        self.draw_popup(f, area);
    }

    #[inline]
    fn draw_popup(&mut self, f: &mut Frame, area: Rect) {
        match self.current_mode {
            // TODO: This is horrid, fix
            Mode::CreateTag => {
                let area =
                    Self::popup_area(area, Constraint::Percentage(30), Constraint::Max(3 * 3));
                self.tag_form.render_widget(f, area);
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
                    Some(i) => crate::mods::tag::SelectedTag::Index(i),
                    None => crate::mods::tag::SelectedTag::None,
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
            Message::AppendMovement(ch) => {
                self.movement_delta.push(ch);
                self.status_line
                    .state
                    .change_hint(self.current_mode, &self.movement_delta);
            }
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
            Message::PropagateEvent(ev) => {
                if let Some(t) = self.tag_form.state.handle_input(&ev) {
                    self.persistent.tags.upsert(t);
                    return Some(Message::ChangeMode(Mode::Normal));
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
            Message::ChangeMode(mode) => {
                self.status_line.state.change_mode(mode);
                self.status_line
                    .state
                    .change_hint(mode, &self.movement_delta);
                // TODO: Generalize
                if matches!(mode, Mode::CreateTag) {
                    self.tag_form.state.reset();
                }
                self.current_mode = mode;
                return Some(Message::ClearCommand);
            }
            Message::Exit => self.should_close = true,
        }
        None
    }
}
