use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};
pub struct FuzzyListWidget<'a> {
    pub title: &'a str,
    pub input: &'a str,
    pub items: Vec<&'a str>,
    pub is_focused: bool,
}

impl<'a> StatefulWidget for FuzzyListWidget<'a> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Split the column into search bar (top) and list (bottom)
        let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(area);

        let border_color = if self.is_focused {
            Color::Yellow
        } else {
            Color::default()
        };

        // 1. Render the Search Input
        Paragraph::new(self.input)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.title)
                    .fg(border_color),
            )
            .render(chunks[0], buf);

        // 2. Render the Results List
        let list_items: Vec<ListItem> =
            self.items.iter().map(|name| ListItem::new(*name)).collect();

        let list = List::new(list_items)
            .block(Block::default().borders(Borders::ALL).title(" Results "))
            .highlight_symbol(">> ")
            .highlight_style(Style::default().bg(Color::Blue).bold());

        // We use the built-in StatefulWidget implementation of List here
        StatefulWidget::render(list, chunks[1], buf, state);
    }
}
