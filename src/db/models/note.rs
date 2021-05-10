use std::path::PathBuf;

use diesel::prelude::*;
use uuid::Uuid;

use crate::db::DbConnection;

use super::{schema::notes, Folder};

#[derive(Queryable, Insertable, Identifiable, AsChangeset, Debug, PartialEq, Eq)]
pub struct Note {
    id: Option<String>,
    parent_id: String,
    pub title: String,
    body: String,
}

impl Note {
    fn new(title: String, parent_id: &str, conn: &DbConnection) -> Result<Self, String> {
        if title.is_empty() {
            return Err("BookLessNote".to_string());
        }
        let note = Self {
            id: Some(Uuid::new_v4().to_string()),
            parent_id: parent_id.into(),
            title,
            body: String::new(),
        };
        diesel::insert_or_ignore_into(notes::table)
            .values(&note)
            .execute(*&conn)
            .map_err(|e| e.to_string())?;

        notes::table
            .filter(notes::title.eq(&note.title))
            .filter(notes::parent_id.eq(&note.parent_id))
            .first(*&conn)
            .map_err(|e| e.to_string())
    }

    fn query(path: &PathBuf, conn: &DbConnection) -> Result<Self, String> {
        let parent_id = Self::get_parent_id(path, false, *&conn)?;
        let title = Self::get_title(&path);
        notes::table
            .filter(notes::title.eq(&title))
            .filter(notes::parent_id.eq(&parent_id))
            .first(*&conn)
            .map_err(|e| e.to_string())
    }

    fn get_title(path: &PathBuf) -> String {
        path.file_name()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or("".to_string())
    }

    fn get_parent_id(path: &PathBuf, parents: bool, conn: &DbConnection) -> Result<String, String> {
        match path.parent() {
            Some(path) if path.parent().is_some() => Ok(Folder::get_id_cow(
                &Folder::query(&path.to_path_buf(), parents, *&conn)?.as_ref(),
            )
            .to_string()),
            _ => Err("BookLessNote".to_string()),
        }
    }

    pub fn make(path: &PathBuf, parents: bool, conn: &DbConnection) -> Result<Self, String> {
        Self::new(
            Self::get_title(&path),
            &Self::get_parent_id(&path, parents, *&conn)?,
            *&conn,
        )
    }

    pub(super) fn list_parent_id(
        parent_id: &str,
        conn: &DbConnection,
    ) -> Result<Vec<Self>, String> {
        Ok(notes::table
            .filter(notes::parent_id.eq(&parent_id))
            .load::<Self>(*&conn)
            .map_err(|e| e.to_string())?)
    }

    pub(super) fn delete_note(&self, conn: &DbConnection) -> Result<usize, String> {
        diesel::delete(
            notes::table
                .filter(notes::title.eq(&self.title))
                .filter(notes::parent_id.eq(&self.parent_id)),
        )
        .execute(*&conn)
        .map_err(|e| e.to_string())
    }

    pub fn cat(path: &PathBuf, conn: &DbConnection) -> Result<String, String> {
        Self::query(&path, *&conn).map(|n| n.body)
    }

    fn update_self(&self, conn: &DbConnection) -> Result<usize, String> {
        diesel::update(notes::table.find(&self.id))
            .set(&*self)
            .execute(*&conn)
            .map_err(|e| e.to_string())
    }

    pub fn update(path: &PathBuf, body: String, conn: &DbConnection) -> Result<usize, String> {
        let mut note = Self::query(&path, *&conn)?;
        note.body = body;
        note.update_self(*&conn)
    }

    pub fn delete(path: &PathBuf, conn: &DbConnection) -> Result<usize, String> {
        Self::query(&path, *&conn)?.delete_note(*&conn)
    }

    pub(super) fn move_self(
        &mut self,
        dest: &PathBuf,
        overwrite: bool,
        parents: bool,
        conn: &DbConnection,
    ) -> Result<usize, String> {
        let mut path = dest.clone();
        path.push(&self.title);
        match Self::query(&path, *&conn) {
            Ok(_) if !overwrite => Err("OverwriteNotAllowed".into()),
            Ok(mut dest_note) => {
                dest_note.body = self.body.clone();
                Ok(dest_note.update_self(*&conn)? + self.delete_note(*&conn)?)
            }
            Err(_) => {
                let parent_id = Self::get_parent_id(&path, parents, *&conn)?;
                self.parent_id = parent_id;
                self.update_self(*&conn)
            }
        }
    }

    pub fn move_note(
        path: &PathBuf,
        dest: &PathBuf,
        overwrite: bool,
        parents: bool,
        conn: &DbConnection,
    ) -> Result<usize, String> {
        let mut note = Self::query(&path, *&conn)?;
        note.move_self(dest, overwrite, parents, &*conn)
    }

    pub(super) fn copy_self(
        self,
        dest: &PathBuf,
        overwrite: bool,
        parents: bool,
        conn: &DbConnection,
    ) -> Result<usize, String> {
        let mut path = dest.clone();
        path.push(&self.title);
        if Self::query(&path, *&conn).is_ok() && !overwrite {
            return Err("OverwriteNotAllowed".into());
        }
        let parent_id = Self::get_parent_id(&path, parents, *&conn)?;
        let mut note = Self::new(self.title, &parent_id, &conn)?;
        note.body = self.body;
        note.update_self(*&conn)
    }

    pub fn copy_note(
        path: &PathBuf,
        dest: &PathBuf,
        overwrite: bool,
        parents: bool,
        conn: &DbConnection,
    ) -> Result<usize, String> {
        let note = Self::query(&path, *&conn)?;
        note.copy_self(dest, overwrite, parents, &*conn)
    }
}
