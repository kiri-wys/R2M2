use ratatui::{
    buffer::Buffer,
    prelude::Rect,
    style::{Color, Stylize as _},
    widgets::{Block, Paragraph, Widget},
};

pub struct InputBox<'text> {
    pub title: &'text str,
    pub buffer: &'text str,
    pub background_color: Color,
    pub foreground_color: Color,
    pub selected: bool,
    pub scroll: u16,
}

impl Widget for &InputBox<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut p = Paragraph::new(self.buffer)
            .block(Block::bordered().title(self.title))
            .fg(self.foreground_color)
            .bg(self.background_color);
        if self.selected {
            p = p.scroll((0, self.scroll));
        }
        p.render(area, buf);
    }
}
