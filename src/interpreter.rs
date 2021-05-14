use rustyline::{error::ReadlineError, Editor};
use structopt::StructOpt;

use crate::{commands::Command, db::DbConnection};

pub struct Interpreter {
    prompt: String,
    conn: DbConnection,
    reader: Editor<()>,
}

#[derive(StructOpt)]
enum InterpreterCommand {
    #[structopt(flatten)]
    Command(Command),
    #[structopt(visible_alias = "q", about = "quit")]
    Quit,
}

impl Interpreter {
    pub fn new(prompt: &str, conn: DbConnection) -> Self {
        let reader = Editor::<()>::new();
        Self {
            prompt: prompt.to_string(),
            conn,
            reader,
        }
    }

    pub fn read_line(&mut self) -> Result<String, ReadlineError> {
        match self.reader.readline(self.prompt.as_str()) {
            Ok(line) => {
                self.reader.add_history_entry(&line);
                Ok(line)
            }
            err => err,
        }
    }

    pub fn run(&mut self) {
        while let Ok(line) = self.read_line() {
            match InterpreterCommand::from_iter_safe(line.split_whitespace()) {
                Ok(InterpreterCommand::Command(c)) => {
                    if let Err(err) = c.execute(&self.conn) {
                        eprintln!("{}", err);
                    }
                }
                Ok(InterpreterCommand::Quit) => {
                    break;
                }
                Err(e) => match e.kind {
                    structopt::clap::ErrorKind::HelpDisplayed => {
                        eprintln!("{}", e.message);
                    }
                    _ => {
                        eprintln!("Error: {:?}", e.kind);
                        eprintln!("Info: {:?}", e.info);
                        eprintln!("\n==============================");
                        InterpreterCommand::clap().print_long_help().unwrap();
                    }
                },
            }
        }
    }
}
