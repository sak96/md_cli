use std::path::PathBuf;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use structopt::StructOpt;
use tui::backend::CrosstermBackend;
use tui::{widgets::ListState, Terminal};

use crate::{
    commands::Command,
    db::{models::Folder, DbConnection},
};

mod views;

pub type TermBackend = Terminal<CrosstermBackend<std::io::Stdout>>;
pub struct AppContext {
    path: PathBuf,
    items_state: ListState,
    conn: DbConnection,
    err_msg: Option<String>,
    terminal: TermBackend,
}

pub enum Item {
    Note(String),
    Folder(String),
}

impl AppContext {
    pub fn new(conn: DbConnection) -> Self {
        let terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))
            .expect("could not create a terminal");
        Self {
            conn,
            terminal,
            path: Default::default(),
            items_state: Default::default(),
            err_msg: Default::default(),
        }
    }

    fn tui_mode(&mut self, on: bool) {
        if on {
            execute!(std::io::stdout(), EnterAlternateScreen, EnableMouseCapture).unwrap();
            enable_raw_mode().unwrap();
            self.terminal.resize(self.terminal.size().unwrap()).unwrap();
        } else {
            disable_raw_mode().unwrap();
            execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
            self.terminal.show_cursor().unwrap();
        }
    }

    pub fn run(&mut self) {
        self.render_loop(&mut terminal);
    }

    fn render_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) {
        self.tui_mode(true);
        loop {
            terminal
                .draw(|rect| {
                    let app_view = views::AppView {
                        items: self.list(),
                        path: self.path.to_string_lossy().into(),
                    };
                    rect.render_stateful_widget(app_view, rect.size(), &mut self.items_state)
                })
                .expect("failed to draw");
            if let Event::Key(key) = event::read().expect("can read events") {
                match key.code {
                    KeyCode::Char('q') => {
                        break;
                    }
                    KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => self.enter(),
                    KeyCode::Backspace | KeyCode::Char('h') | KeyCode::Left => self.back(),
                    KeyCode::Char('k') | KeyCode::Up => self.up(),
                    KeyCode::Char('j') | KeyCode::Down => self.down(),
                    _ => {}
                }
            }
        }
        self.tui_mode(false);
    }

    pub fn list(&mut self) -> Vec<Item> {
        match Folder::list(&self.path, &self.conn) {
            Ok((folders, notes)) => folders
                .into_iter()
                .map(|f| Item::Folder(f.title))
                .chain(notes.into_iter().map(|n| Item::Note(n.title)))
                .collect(),
            Err(msg) => {
                self.err_msg = Some(msg);
                vec![]
            }
        }
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

    fn enter(&mut self) {
        if let Some(selected) = self.items_state.selected() {
            let items = self.list();
            match items.get(selected) {
                Some(Item::Note(title)) => {
                    let mut note = self.path.clone();
                    note.push(title);
                    if let Ok(cmd) = Command::from_iter_safe(vec![
                        "edit".to_string(),
                        note.to_string_lossy().into(),
                    ]) {
                        self.tui_mode(false);
                        if let Err(msg) = cmd.execute(&self.conn) {
                            self.err_msg = Some(msg)
                        }
                        self.tui_mode(true);
                    }
                }
                Some(Item::Folder(title)) => {
                    self.path.push(title);
                    self.items_state.select(None);
                }
                None => {}
            }
        }
    }

    fn back(&mut self) {
        self.path.pop();
    }
}
