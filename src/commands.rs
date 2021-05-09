use crate::db::{models::*, DbConnection};
use structopt::{clap::AppSettings, StructOpt};

#[derive(StructOpt, Debug)]
#[structopt(about, rename_all = "lower",global_settings(&[AppSettings::VersionlessSubcommands, AppSettings::NoBinaryName, AppSettings::DisableVersion]))]
pub enum Command {
    #[structopt(visible_alias = "ls", about = "list book")]
    List {
        #[structopt(default_value = "/")]
        path: std::path::PathBuf,
    },
    #[structopt(visible_alias = "mb", about = "make book")]
    MkBook {
        #[structopt(short = "p")]
        parents: bool,
        path: std::path::PathBuf,
    },
    #[structopt(visible_alias = "rb", about = "remove book")]
    RmBook {
        #[structopt(short = "r")]
        recursive: bool,
        path: std::path::PathBuf,
    },
    #[structopt(visible_alias = "mn", about = "make notes")]
    MkNote { path: std::path::PathBuf },
    #[structopt(visible_alias = "rn", about = "remove notes")]
    RmNote { path: std::path::PathBuf },
    #[structopt(about = "output not to file or stdout")]
    Cat {
        path: std::path::PathBuf,
        dest: Option<std::path::PathBuf>,
    },
    #[structopt(visible_alias = "mv", about = "update note with give file")]
    Update {
        path: std::path::PathBuf,
        dest: Option<std::path::PathBuf>,
    },
    #[structopt(visible_alias = "mv", about = "move note")]
    Move {
        src: std::path::PathBuf,
        dest: std::path::PathBuf,
    },
    #[structopt(visible_alias = "cp", about = "move note")]
    Copy {
        src: std::path::PathBuf,
        dest: std::path::PathBuf,
    },
}

impl Command {
    pub fn execute(&self, connection: &DbConnection) -> Result<(), String> {
        match &self {
            Command::List { path } => {
                let (folders, _notes) = Folder::list(&path, &connection)?;
                for folder in folders {
                    println!("{}", folder.title);
                }
            }
            Command::MkBook { parents, path } => {
                Folder::make(&path, *parents, &connection)?;
                print!("{} successfully created", path.to_string_lossy());
            }
            Command::RmBook { recursive, path } => {
                print!(
                    "{} successfully delete\n{} rows delete",
                    path.to_string_lossy(),
                    Folder::delete(path, *recursive, &connection)?
                );
            }
            _ => {
                unimplemented!()
            }
        }
        Ok(())
    }
}
