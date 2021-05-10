#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

mod commands;
pub mod db;

use commands::Command;
use structopt::StructOpt;

fn main() {
    let args = ::std::env::args();

    if args.len() < 2 {
        eprintln!("interactive mode not yet supported");
    } else {
        match Command::from_iter_safe(args.into_iter().skip(1)) {
            Ok(c) => match db::establish_connection() {
                Ok(connection) => {
                    if let Err(err) = c.execute(&connection) {
                        eprintln!("{}", err);
                    }
                }
                Err(msg) => {
                    eprintln!("{}", msg);
                }
            },
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
