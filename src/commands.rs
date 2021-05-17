use std::{
    fs::OpenOptions,
    io::{stdin, stdout, Read, Write},
};

use crate::db::{models::*, DbConnection};
use structopt::{clap::AppSettings, StructOpt};
use uuid::Uuid;

#[derive(StructOpt, Debug)]
#[structopt(about, rename_all = "lower",global_settings(&[AppSettings::VersionlessSubcommands, AppSettings::NoBinaryName, AppSettings::DisableVersion]))]
pub enum Command {
    #[structopt(visible_alias = "ls", about = "list book")]
    List {
        #[structopt(skip)]
        indent: String,
        #[structopt(short = "r")]
        recursive: bool,
        #[structopt(default_value = "/")]
        path: std::path::PathBuf,
    },
    #[structopt(visible_alias = "mk", about = "remove book (path ends with /) or note")]
    Make {
        #[structopt(short = "p")]
        parents: bool,
        path: std::path::PathBuf,
    },
    #[structopt(visible_alias = "rm", about = "remove book (path ends with /) or note")]
    Remove {
        #[structopt(short = "r")]
        recursive: bool,
        path: std::path::PathBuf,
    },
    #[structopt(about = "output note to file or stdout")]
    Cat {
        note: std::path::PathBuf,
        out_file: Option<std::path::PathBuf>,
    },
    #[structopt(about = "update note with give file or stdin")]
    Update {
        note: std::path::PathBuf,
        in_file: Option<std::path::PathBuf>,
    },
    #[structopt(about = "edit note in an editor")]
    Edit { note: std::path::PathBuf },
    #[structopt(visible_alias = "mv", about = "move note")]
    Move {
        #[structopt(short = "p")]
        parents: bool,
        #[structopt(short = "o")]
        overwrite: bool,
        src: std::path::PathBuf,
        dest_book: std::path::PathBuf,
    },
    #[structopt(visible_alias = "cp", about = "copy note")]
    Copy {
        #[structopt(short = "p")]
        parents: bool,
        #[structopt(short = "o")]
        overwrite: bool,
        src: std::path::PathBuf,
        dest_book: std::path::PathBuf,
    },
}

impl Command {
    pub fn execute(&self, connection: &DbConnection) -> Result<String, String> {
        // dbg!(&self);
        let output;
        match &self {
            Command::List {
                indent,
                recursive,
                path,
            } => {
                let mut buffer = String::new();
                let (folders, notes) = Folder::list(&path, &connection)?;
                let mut count = folders.len() + notes.len();
                let mut connector = || {
                    count -= 1;
                    if count > 0 {
                        "├─"
                    } else {
                        "└─"
                    }
                };
                for note in notes {
                    buffer.push_str(&format!("{}{}{}\n", indent, connector(), note.title));
                }
                for folder in folders {
                    let connector = connector();
                    buffer.push_str(&format!("{}{}{}/\n", indent, connector, folder.title));
                    if *recursive {
                        let mut path = path.clone();
                        path.push(folder.title);
                        buffer.push_str(
                            &Command::List {
                                indent: format!(
                                    "{}{}",
                                    indent,
                                    if connector == "├─" { "│ " } else { "  " }
                                ),
                                recursive: true,
                                path,
                            }
                            .execute(&connection)?,
                        );
                    }
                }
                output = buffer;
            }
            Command::Make { parents, path } => {
                if path.to_string_lossy().ends_with("/") {
                    Folder::make(&path, *parents, &connection)?;
                } else {
                    Note::make(&path, *parents, &connection)?;
                }
                output = format!("{} successfully created", path.to_string_lossy());
            }
            Command::Remove { recursive, path } => {
                let rows = if path.to_string_lossy().ends_with("/") {
                    Folder::delete(path, *recursive, &connection)?
                } else {
                    Note::delete(path, &connection)?
                };
                output = format!(
                    "{} successfully delete\n{} rows delete",
                    path.to_string_lossy(),
                    rows
                );
            }
            Command::Cat {
                note,
                out_file: dest,
            } => {
                let mut writer: Box<dyn Write> = match dest {
                    Some(path) => Box::new(
                        OpenOptions::new()
                            .write(true)
                            .create(true)
                            .open(&path)
                            .map_err(|e| e.to_string())?,
                    ),
                    None => Box::new(stdout()),
                };
                writer
                    .write_all(Note::cat(&note, &connection)?.as_bytes())
                    .map_err(|e| e.to_string())?;
                output = String::new();
            }
            Command::Edit { note } => {
                let mut temp_file = std::env::temp_dir();
                temp_file.push(&format!(
                    "tmp_{}_{}_{}",
                    structopt::clap::crate_name!(),
                    Uuid::new_v4().to_string(),
                    note.file_name()
                        .unwrap_or(Default::default())
                        .to_string_lossy()
                ));
                Command::Cat {
                    note: note.clone(),
                    out_file: Some(temp_file.clone()),
                }
                .execute(&connection)?;
                edit::edit_file(&temp_file).map_err(|e| e.to_string())?;
                Command::Update {
                    note: note.clone(),
                    in_file: Some(temp_file.clone()),
                }
                .execute(&connection)?;
                std::fs::remove_file(temp_file).map_err(|e| e.to_string())?;
                output = String::new();
            }
            Command::Update { note, in_file: src } => {
                let mut reader: Box<dyn Read> = match src {
                    Some(path) => Box::new(
                        OpenOptions::new()
                            .read(true)
                            .open(&path)
                            .map_err(|e| e.to_string())?,
                    ),
                    None => Box::new(stdin()),
                };
                let mut body = String::new();
                reader
                    .read_to_string(&mut body)
                    .map_err(|e| e.to_string())?;
                let rows = Note::update(&note, body, &connection)?;
                output = format!(
                    "{} successfully created\n {} rows effected",
                    note.to_string_lossy(),
                    rows
                );
            }
            Command::Copy {
                parents,
                overwrite,
                src,
                dest_book,
            } => {
                let rows = Note::copy_note(&src, &dest_book, *overwrite, *parents, &connection)?;
                output = format!("copy successful\n {} rows effected", rows);
            }
            Command::Move {
                parents,
                overwrite,
                src,
                dest_book,
            } => {
                let rows = Note::move_note(&src, &dest_book, *overwrite, *parents, &connection)?;
                output = format!("move successful\n {} rows effected", rows);
            }
        }
        Ok(output)
    }
}
