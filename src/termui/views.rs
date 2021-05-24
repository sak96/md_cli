use std::{collections::HashMap, path::PathBuf};

use crossterm::event::KeyCode;
use string_template::Template;
use structopt::StructOpt;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

use crate::{
    commands::Command,
    db::{models::Folder, DbConnection},
};

pub enum ActiveElement {
    FolderView,
    Interpreter,
}

impl Default for ActiveElement {
    fn default() -> Self {
        Self::FolderView
    }
}

#[derive(StructOpt)]
pub enum TuiCommand {
    #[structopt(flatten)]
    Command(Command),
    #[structopt(visible_alias = "q", about = "quit")]
    Quit,
}

pub enum Item {
    Note(String),
    Folder(String),
}

impl Item {
    pub fn get_color(&self) -> Color {
        match self {
            Self::Note(_) => Color::Blue,
            Self::Folder(_) => Color::Green,
        }
    }

    pub fn get_name(self) -> String {
        match self {
            Self::Note(path) => path,
            Self::Folder(mut path) => {
                path.push('/');
                path
            }
        }
    }
}

pub enum Return {
    Command(TuiCommand),
    State(ActiveElement),
    Error(String),
    Pass,
}

#[derive(Default)]
pub struct FolderView {
    path: PathBuf,
    items_state: ListState,
}

impl FolderView {
    pub fn get_context(&self, conn: &DbConnection) -> Context {
        Context {
            path: self.path.clone(),
            note: self.get_selected_item(conn),
        }
    }

    fn get_selected_item(&self, conn: &DbConnection) -> Option<Item> {
        self.list(conn)
            .ok()?
            .into_iter()
            .skip(self.items_state.selected()?)
            .next()
    }

    pub fn list(&self, conn: &DbConnection) -> Result<Vec<Item>, String> {
        let (folders, notes) = Folder::list(&self.path, conn)?;
        Ok(folders
            .into_iter()
            .map(|f| Item::Folder(f.title))
            .chain(notes.into_iter().map(|n| Item::Note(n.title)))
            .collect())
    }

    fn up(&mut self) {
        if let Some(selected) = self.items_state.selected() {
            if selected != 0 {
                self.items_state.select(Some(selected - 1));
            }
        }
    }

    fn down(&mut self) {
        if let Some(selected) = self.items_state.selected() {
            self.items_state.select(Some(selected + 1));
        }
    }

    fn enter(&mut self, selected_item: Option<Item>) -> Option<PathBuf> {
        match selected_item {
            Some(Item::Note(title)) => {
                let mut note = self.path.clone();
                note.push(title);
                return Some(note);
            }
            Some(Item::Folder(title)) => {
                self.path.push(title);
                self.items_state.select(None);
            }
            None => {}
        }
        None
    }

    fn back(&mut self) {
        self.path.pop();
    }

    pub fn handle_events(&mut self, key: KeyCode, conn: &DbConnection) -> Return {
        match key {
            KeyCode::Char('q') => return Return::Command(TuiCommand::Quit),
            KeyCode::Backspace | KeyCode::Char('h') | KeyCode::Left => self.back(),
            KeyCode::Char('k') | KeyCode::Up => self.up(),
            KeyCode::Char('j') | KeyCode::Down => self.down(),
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
                if let Some(note) = self.enter(self.get_selected_item(&conn)) {
                    return Return::Command(TuiCommand::Command(Command::Edit { note }));
                }
            }
            KeyCode::Char(':') | KeyCode::Tab => return Return::State(ActiveElement::Interpreter),
            _ => {}
        }
        Return::Pass
    }
}

impl StatefulWidget for &mut FolderView {
    fn render(self, area: Rect, buf: &mut Buffer, (items, is_active): &mut Self::State) {
        let block = Block::default()
            .borders(Borders::ALL)
            .style({
                let style = Style::default();
                if *is_active {
                    style.fg(Color::Yellow)
                } else {
                    style.fg(Color::White)
                }
            })
            .title(self.path.to_str().unwrap_or(""));
        match self.items_state.selected() {
            _ if items.is_empty() => {
                self.items_state.select(None);
            }
            Some(selected) if selected >= items.len() => {
                self.items_state.select(Some(items.len() - 1));
            }
            None => {
                self.items_state.select(Some(0));
            }
            _ => {}
        }
        let list = List::new(
            items
                .drain(..)
                .map(|item| {
                    ListItem::new(Spans::from(vec![{
                        let color = item.get_color();
                        Span::styled(item.get_name(), Style::default().fg(color))
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
        StatefulWidget::render(list, area, buf, &mut self.items_state);
    }

    type State = (Vec<Item>, bool);
}

#[derive(Default)]
pub struct PopUpView {
    pub msgs: Vec<Result<String, String>>,
}

impl PopUpView {
    pub fn push(&mut self, value: Result<String, String>) {
        self.msgs.push(value)
    }
}

impl Widget for &mut PopUpView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("popup");
        let list = List::new(
            self.msgs
                .iter()
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

#[derive(Default)]
pub struct Interpreter {
    input: String,
}

pub struct Context {
    pub path: PathBuf,
    pub note: Option<Item>,
}

impl Interpreter {
    pub fn handle_events(&mut self, key: KeyCode, context: Context) -> Return {
        match key {
            KeyCode::Enter => {
                let template = Template::new(&self.input.drain(..).collect::<String>());
                let mut subs = HashMap::new();

                let parent = context.path.to_str().unwrap_or("");
                let child = format!(
                    "{}/{}",
                    parent,
                    context
                        .note
                        .unwrap_or(Item::Folder(String::new()))
                        .get_name()
                );
                subs.insert("parent", parent);
                subs.insert("P", parent);
                subs.insert("child", &child);
                subs.insert("C", &child);

                let output = template.render(&subs);
                let args = match shellwords::split(&output) {
                    Ok(args) => args,
                    Err(err) => {
                        return Return::Error(err.to_string());
                    }
                };
                match TuiCommand::from_iter_safe(args) {
                    Ok(tui_cmd) => return Return::Command(tui_cmd),
                    Err(e) => return Return::Error(e.to_string()),
                }
            }
            KeyCode::Esc | KeyCode::Tab => return Return::State(ActiveElement::FolderView),
            KeyCode::Char(c) => self.input.push(c),
            KeyCode::Backspace => {
                self.input.pop();
            }
            _ => {}
        }
        Return::Pass
    }
}

impl StatefulWidget for &mut Interpreter {
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
            "{}|Esc: to stop editing|Enter: to execute|{{c/CHILD}}: child|{{p/PARENT}}: parent",
            structopt::clap::crate_name!()
        )));
        input.render(area, buf);
    }

    type State = bool;
}
