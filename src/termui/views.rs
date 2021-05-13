use tui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
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
        StatefulWidget::render(list, area, buf, state);
    }

    type State = ListState;
}

pub struct PopUpView {
    pub msg: String,
}

impl Widget for PopUpView {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        let percent_y = 20;
        let percent_x = 50;
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_y) / 2),
                    Constraint::Percentage(percent_y),
                    Constraint::Percentage((100 - percent_y) / 2),
                ]
                .as_ref(),
            )
            .split(area);

        let popup = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_x) / 2),
                    Constraint::Percentage(percent_x),
                    Constraint::Percentage((100 - percent_x) / 2),
                ]
                .as_ref(),
            )
            .split(popup_layout[1])[1];

        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("popup");
        Paragraph::new(self.msg)
            .alignment(Alignment::Center)
            .block(block)
            .render(popup, buf);
    }
}
