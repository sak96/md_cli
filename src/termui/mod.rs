use std::path::PathBuf;

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use structopt::StructOpt;
use tui::backend::CrosstermBackend;
use tui::{widgets::ListState, Terminal};

use crate::{
    commands::Command,
    db::{models::Folder, DbConnection},
};

mod views;

pub struct AppContext {
    path: PathBuf,
    items_state: ListState,
    conn: DbConnection,
    err_msg: Option<String>,
    clear: bool,
}

pub enum Item {
    Note(String),
    Folder(String),
}

impl AppContext {
    pub fn new(conn: DbConnection) -> Self {
        Self {
            conn,
            path: Default::default(),
            items_state: Default::default(),
            err_msg: Default::default(),
            clear: Default::default(),
        }
    }

    pub fn run(&mut self) {
        // TODO: fix these
        enable_raw_mode().expect("can run in raw mode");
        let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))
            .expect("could not create a terminal");
        terminal.clear().expect("could not clear terminal");
        self.render_loop(&mut terminal);
        disable_raw_mode().expect("can run in raw mode");
        terminal.show_cursor().expect("can show cursor");
    }

    fn render_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) {
        loop {
            if self.clear {
                terminal.clear().expect("could not clear terminal");
                self.clear = false;
            }
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
                        if let Err(msg) = cmd.execute(&self.conn) {
                            self.err_msg = Some(msg)
                        }
                    }
                    self.clear = true;
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
