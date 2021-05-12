#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

mod commands;
pub mod db;

#[cfg(feature = "termui")]
mod termui;

use commands::Command;
use structopt::StructOpt;

fn main() {
    let args = ::std::env::args();
    match db::establish_connection() {
        Ok(connection) => {
            if args.len() < 2 {
                #[cfg(not(feature = "termui"))]
                eprintln!("use --features termui to enable tui.");
                #[cfg(feature = "termui")]
                termui::AppContext::new(connection).run()
            } else {
                match Command::from_iter_safe(args.into_iter().skip(1)) {
                    Ok(c) => {
                        if let Err(err) = c.execute(&connection) {
                            eprintln!("{}", err);
                        }
                    }
                    Err(e) => match e.kind {
                        structopt::clap::ErrorKind::HelpDisplayed => {
                            eprintln!("{}", e.message);
                        }
                        _ => {
                            eprintln!("Error: {:?}", e.kind);
                            eprintln!("Info: {:?}", e.info);
                            eprintln!("\n==============================");
                            Command::clap().print_long_help().unwrap();
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
