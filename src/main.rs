#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

mod commands;
pub mod db;
mod interpreter;

#[cfg(feature = "termui")]
mod termui;

use commands::Command;
use structopt::StructOpt;

#[derive(StructOpt)]
pub enum SingleCommand {
    #[structopt(flatten)]
    Single(Command),
    Tui,
}

fn main() {
    let args = ::std::env::args();
    match db::establish_connection() {
        Ok(connection) => {
            if args.len() < 2 {
                let prompt = format!("{}% ", structopt::clap::crate_name!());
                interpreter::Interpreter::new(&prompt, connection).run();
            } else {
                match SingleCommand::from_iter_safe(args.into_iter().skip(1)) {
                    Ok(SingleCommand::Single(c)) => {
                        if let Err(err) = c.execute(&connection) {
                            eprintln!("{}", err);
                        }
                    }
                    Ok(SingleCommand::Tui) => {
                        #[cfg(not(feature = "termui"))]
                        eprintln!("use --features termui to enable tui.");
                        #[cfg(feature = "termui")]
                        termui::AppContext::new(connection).run()
                    }
                    Err(e) => match e.kind {
                        structopt::clap::ErrorKind::HelpDisplayed => {
                            eprintln!("{}", e.message);
                        }
                        _ => {
                            eprintln!("Error: {:?}", e.kind);
                            eprintln!("Info: {:?}", e.info);
                            eprintln!("\n==============================");
                            SingleCommand::clap().print_long_help().unwrap();
                        }
                    },
                }
            }
        }
        Err(msg) => {
            eprintln!("Db Error:{}", msg);
        }
    }
}
