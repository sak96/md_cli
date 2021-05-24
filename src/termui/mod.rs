use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use tui::Terminal;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
};

use crate::db::DbConnection;

use self::views::{ActiveElement, FolderView, Interpreter, PopUpView, TuiCommand};

mod views;

pub type TermBackend = Terminal<CrosstermBackend<std::io::Stdout>>;
pub struct AppContext {
    conn: DbConnection,
    interpreter: Interpreter,
    folder: FolderView,
    popup: PopUpView,
    state: ActiveElement,
    terminal: TermBackend,
}

impl AppContext {
    pub fn new(conn: DbConnection) -> Self {
        let terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))
            .expect("could not create a terminal");
        Self {
            conn,
            terminal,
            state: Default::default(),
            interpreter: Default::default(),
            popup: Default::default(),
            folder: Default::default(),
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
        let AppContext {
            terminal,
            state,
            folder,
            popup,
            interpreter,
            conn,
        } = self;
        let items = match folder.list(&conn) {
            Ok(items) => items,
            Err(msg) => {
                popup.push(Err(msg));
                vec![]
            }
        };
        let is_folder_view = matches!(state, ActiveElement::FolderView);
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
                rect.render_stateful_widget(folder, chucks_2[0], &mut (items, is_folder_view));
                rect.render_widget(popup, chucks_2[1]);
                rect.render_stateful_widget(interpreter, chunks[1], &mut !is_folder_view);
            })
            .expect("failed to draw");
    }

    pub fn run(&mut self) {
        self.tui_mode(true);
        let mut running = true;
        while running {
            self.render();
            if let Event::Key(key) = event::read().expect("can read events") {
                match match self.state {
                    ActiveElement::FolderView => self.folder.handle_events(key.code, &self.conn),
                    ActiveElement::Interpreter => self
                        .interpreter
                        .handle_events(key.code, self.folder.get_context(&self.conn)),
                } {
                    views::Return::Command(TuiCommand::Quit) => running = false,
                    views::Return::Command(TuiCommand::Command(cmd)) => {
                        self.tui_mode(false);
                        self.popup.push(cmd.execute(&self.conn));
                        self.tui_mode(true);
                    }
                    views::Return::State(s) => self.state = s,
                    views::Return::Error(e) => self.popup.push(Err(e)),
                    views::Return::Pass => {}
                }
            }
        }
        self.tui_mode(false);
    }
}
