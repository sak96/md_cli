use std::path::PathBuf;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use structopt::StructOpt;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
};
use tui::{widgets::ListState, Terminal};

use crate::{
    commands::Command,
    db::{models::Folder, DbConnection},
};

mod views;

#[derive(StructOpt)]
enum TuiCommand {
    #[structopt(flatten)]
    Command(Command),
    #[structopt(visible_alias = "q", about = "quit")]
    Quit,
}

pub type TermBackend = Terminal<CrosstermBackend<std::io::Stdout>>;
pub struct AppContext {
    path: PathBuf,
    items_state: ListState,
    conn: DbConnection,
    err_msg: Vec<Result<String, String>>,
    input: String,
    state: ActiveElement,
    terminal: TermBackend,
}

enum ActiveElement {
    FolderView,
    Interpreter,
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
            state: ActiveElement::FolderView,
            path: Default::default(),
            input: Default::default(),
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

    pub fn render(&mut self) {
        let items = self.list();
        {
            let AppContext {
                path,
                ref mut items_state,
                conn: _conn,
                ref mut err_msg,
                input,
                state,
                ref mut terminal,
            } = self;
            terminal
                .draw(|rect| {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(2)
                        .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
                        .split(rect.size());

                    let chucks_2 = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Ratio(1, 3), Constraint::Min(1)].as_ref())
                        .split(chunks[0]);

                    let is_folder_view = matches!(state, ActiveElement::FolderView);
                    if err_msg.len() > 20 {
                        err_msg.drain(20..);
                    }
                    let popup = views::PopUpView {
                        msg: err_msg.as_ref(),
                    };
                    let interpreter = views::Interpreter {
                        input: input.clone(),
                    };
                    let app_view = views::AppView {
                        items,
                        path: path.to_string_lossy().to_string(),
                        active: is_folder_view.clone(),
                    };
                    rect.render_stateful_widget(app_view, chucks_2[0], items_state);
                    rect.render_widget(popup, chucks_2[1]);
                    rect.render_stateful_widget(interpreter, chunks[1], &mut !is_folder_view);
                })
                .expect("failed to draw");
        }
    }

    pub fn handle_events(&mut self) -> bool {
        if let Event::Key(key) = event::read().expect("can read events") {
            match self.state {
                ActiveElement::FolderView => match key.code {
                    KeyCode::Char('q') => return false,
                    KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => self.enter(),
                    KeyCode::Backspace | KeyCode::Char('h') | KeyCode::Left => self.back(),
                    KeyCode::Char('k') | KeyCode::Up => self.up(),
                    KeyCode::Char('j') | KeyCode::Down => self.down(),
                    KeyCode::Char(':') | KeyCode::Tab => self.state = ActiveElement::Interpreter,
                    _ => {}
                },
                ActiveElement::Interpreter => match key.code {
                    KeyCode::Enter => {
                        match TuiCommand::from_iter_safe(
                            self.input
                                .drain(..)
                                .collect::<String>()
                                .split_ascii_whitespace(),
                        ) {
                            Ok(TuiCommand::Quit) => return false,
                            Ok(TuiCommand::Command(cmd)) => {
                                self.tui_mode(false);
                                self.err_msg.push(cmd.execute(&self.conn));
                                self.tui_mode(true);
                            }
                            Err(e) => {
                                if matches!(e.kind, structopt::clap::ErrorKind::HelpDisplayed) {
                                    self.err_msg.push(Ok(e.message))
                                } else {
                                    self.err_msg
                                        .push(Err(format!("{:?} error ({:?})", e.kind, e.info)))
                                }
                            }
                        }
                    }
                    KeyCode::Char(c) => self.input.push(c),
                    KeyCode::Backspace => {
                        self.input.pop();
                    }
                    KeyCode::Esc | KeyCode::Tab => self.state = ActiveElement::FolderView,
                    _ => {}
                },
            }
        }
        return true;
    }

    pub fn run(&mut self) {
        self.tui_mode(true);
        loop {
            self.render();
            if !self.handle_events() {
                break;
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
                self.err_msg.push(Err(msg));
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
                        self.err_msg.push(cmd.execute(&self.conn));
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
