use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

use super::Item;

pub struct AppView {
    pub items: Vec<Item>,
    pub path: String,
    pub active: bool,
}

impl StatefulWidget for AppView {
    fn render(self, area: Rect, buf: &mut Buffer, list_state: &mut Self::State) {
        let block = Block::default()
            .borders(Borders::ALL)
            .style({
                let style = Style::default();
                if self.active {
                    style.fg(Color::Yellow)
                } else {
                    style.fg(Color::White)
                }
            })
            .title(self.path);
        match list_state.selected() {
            _ if self.items.is_empty() => {
                list_state.select(None);
            }
            Some(selected) if selected >= self.items.len() => {
                list_state.select(Some(self.items.len() - 1));
            }
            None => {
                list_state.select(Some(0));
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
        StatefulWidget::render(list, area, buf, list_state);
    }

    type State = ListState;
}

pub struct PopUpView<'a> {
    pub msg: &'a [Result<String, String>],
}

impl Widget for PopUpView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("popup");
        let list = List::new(
            self.msg
                .into_iter()
                .rev()
                .map(|item| {
                    ListItem::new(match item {
                        Err(msg) => Text::styled(msg, Style::default().fg(Color::Red)),
                        Ok(msg) => Text::styled(msg, Style::default().fg(Color::White)),
                    })
                })
                .collect::<Vec<_>>(),
        )
        .block(block);
        Widget::render(list, area, buf);
    }
}

pub struct Interpreter {
    pub input: String,
}

impl StatefulWidget for Interpreter {
    fn render(self, area: Rect, buf: &mut Buffer, input_mode: &mut Self::State) {
        let input = Paragraph::new(Spans::from({
            let input: &str = self.input.as_ref();
            let mut spans = vec![Span::raw(input)];
            if *input_mode {
                spans.push(Span::styled(
                    "_".to_owned(),
                    Style::default()
                        .add_modifier(Modifier::UNDERLINED)
                        .fg(Color::Yellow),
                ))
            }
            spans
        }))
        .style({
            let mut style = Style::default();
            if *input_mode {
                style = style.fg(Color::Yellow)
            }
            style
        })
        .block(Block::default().borders(Borders::ALL).title(format!(
            "{}|Esc: to stop editing|Enter: to execute|",
            structopt::clap::crate_name!()
        )));
        input.render(area, buf);
    }

    type State = bool;
}
