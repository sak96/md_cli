use std::{
    fs::OpenOptions,
    io::{stdin, stdout, Read, Write},
    path::PathBuf,
    process,
    time::SystemTime,
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
    MkNote {
        #[structopt(short = "p")]
        parents: bool,
        path: std::path::PathBuf,
    },
    #[structopt(visible_alias = "rn", about = "remove notes")]
    RmNote { path: std::path::PathBuf },
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
    Edit {
        #[structopt(short = "e", env = "EDITOR")]
        editor: Option<std::path::PathBuf>,
        note: std::path::PathBuf,
    },
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
    pub fn execute(&self, connection: &DbConnection) -> Result<(), String> {
        // dbg!(&self);
        match &self {
            Command::List {
                indent,
                recursive,
                path,
            } => {
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
                    println!("{}{}{}", indent, connector(), note.title);
                }
                for folder in folders {
                    let connector = connector();
                    println!("{}{}{}/", indent, connector, folder.title);
                    if *recursive {
                        let mut path = path.clone();
                        path.push(folder.title);
                        Command::List {
                            indent: format!(
                                "{}{}",
                                indent,
                                if connector == "├─" { "│ " } else { "  " }
                            ),
                            recursive: true,
                            path,
                        }
                        .execute(&connection)?;
                    }
                }
            }
            Command::MkBook { parents, path } => {
                Folder::make(&path, *parents, &connection)?;
                println!("{} successfully created", path.to_string_lossy());
            }
            Command::RmBook { recursive, path } => {
                println!(
                    "{} successfully delete\n{} rows delete",
                    path.to_string_lossy(),
                    Folder::delete(path, *recursive, &connection)?
                );
            }
            Command::MkNote { parents, path } => {
                Note::make(&path, *parents, &connection)?;
                println!("{} successfully created", path.to_string_lossy());
            }
            Command::RmNote { path } => {
                println!(
                    "{} successfully delete\n{} rows delete",
                    path.to_string_lossy(),
                    Note::delete(path, &connection)?
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
            }
            Command::Edit { editor, note } => {
                if let Some(editor) = editor {
                    let mut temp_file = std::env::temp_dir();
                    temp_file.push(&format!(
                        "tmp_{}_{}_{}",
                        structopt::clap::crate_name!(),
                        Uuid::new_v4().to_string(),
                        note.file_name()
                            .unwrap_or(Default::default())
                            .to_string_lossy()
                    ));
                    {
                        Command::Cat {
                            note: note.clone(),
                            out_file: Some(temp_file.clone()),
                        }
                        .execute(&connection)?;
                    }
                    let mut command = process::Command::new(editor);
                    command.arg(&temp_file);
                    let get_last_modified = |path: &PathBuf| -> std::io::Result<SystemTime> {
                        std::fs::metadata(&path)?.modified()
                    };
                    let last_modified = get_last_modified(&temp_file).map_err(|e| e.to_string())?;
                    {
                        let mut handle = command.spawn().map_err(|e| e.to_string())?;
                        handle.wait().map_err(|e| e.to_string())?;
                    }
                    if last_modified < get_last_modified(&temp_file).map_err(|e| e.to_string())? {
                        {
                            Command::Update {
                                note: note.clone(),
                                in_file: Some(temp_file.clone()),
                            }
                            .execute(&connection)?;
                        }
                    } else {
                        println!("Nothing modified")
                    }
                    std::fs::remove_file(temp_file).map_err(|e| e.to_string())?;
                } else {
                    Err("EditorUndefined")?
                }
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
                println!(
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
                println!("copy successful\n {} rows effected", rows);
            }
            Command::Move {
                parents,
                overwrite,
                src,
                dest_book,
            } => {
                let rows = Note::move_note(&src, &dest_book, *overwrite, *parents, &connection)?;
                println!("move successful\n {} rows effected", rows);
            }
        }
        Ok(())
    }
}
