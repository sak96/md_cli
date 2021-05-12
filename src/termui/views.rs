use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget},
};

use super::Item;

pub struct AppView {
    pub items: Vec<Item>,
    pub path: String,
}

impl StatefulWidget for AppView {
    fn render(
        self,
        area: tui::layout::Rect,
        buf: &mut tui::buffer::Buffer,
        state: &mut Self::State,
    ) {
        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title(self.path);
        match state.selected() {
            _ if self.items.is_empty() => {
                state.select(None);
            }
            Some(selected) if selected >= self.items.len() => {
                state.select(Some(self.items.len() - 1));
            }
            None => {
                state.select(Some(0));
            }
            _ => {}
        }
        let list = List::new(
            self.items
                .into_iter()
                .map(|item| {
                    ListItem::new(Spans::from(vec![match item {
                        Item::Note(title) => Span::styled(title, Style::default().fg(Color::Blue)),
                        Item::Folder(mut title) => {
                            title.push_str("/");
                            Span::styled(title, Style::default().fg(Color::Green))
                        }
                    }]))
                })
                .collect::<Vec<_>>(),
        )
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );
        list.render(area, buf, state);
    }

    type State = ListState;
}
