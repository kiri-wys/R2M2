use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize as _},
    text::{Line, Span},
    widgets::{StatefulWidget, Widget},
};

use crate::app::Mode;

#[derive(Debug, Default)]
pub struct StatusLine {
    pub widget: StatusLineWidget,
    pub state: StatusLineState,
}
impl StatusLine {
    pub fn render_widget(&mut self, frame: &mut Frame, area: Rect) {
        let &mut Self {
            ref widget,
            ref mut state,
        } = self;
        frame.render_stateful_widget(widget, area, state);
    }
}

#[derive(Debug, Default)]
pub struct StatusLineWidget;
#[derive(Debug, Default)]
pub struct StatusLineState {
    pub background_color: Color,

    left: Line<'static>,
    right: Line<'static>,
}
impl StatusLineState {
    pub fn change_mode(&mut self, mode: Mode) {
        let text = mode.str_repr();
        let mode_color = mode.color_repr();

        self.left = Line::from(vec![
            Span::styled(
                text,
                Style::default()
                    .bg(mode_color)
                    .fg(self.background_color)
                    .bold(),
            ),
            Span::styled(
                "",
                Style::default().fg(mode_color).bg(self.background_color),
            ),
        ])
        .bg(self.background_color);
    }
    pub fn change_hint(&mut self, mode: Mode, movement_delta: &str) {
        let text = if !movement_delta.is_empty() {
            movement_delta.to_string()
        } else {
            match mode {
                Mode::Normal => "Press ? for help, 'q' to quit",
                Mode::CreateTag => "Creating new tag, ESC to go back.",
                Mode::ShowTags => "Listing created tags, 'e' to edit, 'q' or ESC to go back",
                Mode::Insert => "Inserting tag into selected mod, ESC to go back",
            }
            .to_string()
        };
        let mode_color = mode.color_repr();

        self.right = Line::from(Span::styled(text, Style::default().fg(mode_color).bold()))
            .alignment(ratatui::layout::Alignment::Right)
            .bg(self.background_color);
    }
}

impl StatefulWidget for &StatusLineWidget {
    type State = StatusLineState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let [left, right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(1), Constraint::Min(1)])
            .areas(area);

        (&state.left).render(left, buf);
        (&state.right).render(right, buf);
    }
}
