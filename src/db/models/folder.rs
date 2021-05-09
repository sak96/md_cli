use std::{borrow::Cow, path::PathBuf};

use diesel::prelude::*;

use super::{schema::folders, Note};
use crate::db::{DbConnection, DieselStringError};
use uuid::Uuid;

#[derive(Queryable, Insertable, Identifiable, Debug, PartialEq, Eq)]
pub struct Folder {
    id: Option<String>,
    pub title: String,
    parent_id: String,
}

impl Folder {
    fn new(title: String, parent: Option<Self>, conn: &DbConnection) -> Result<Self, String> {
        if title.is_empty() {
            return Err("NamelessFolder".to_string());
        }
        let folder = Self {
            id: Some(Uuid::new_v4().to_string()),
            parent_id: Self::get_id_cow(&parent.as_ref()).to_string(),
            title,
        };

        diesel::insert_or_ignore_into(folders::table)
            .values(&folder)
            .execute(*&conn)
            .map_err(|e| e.to_string())?;

        folders::table
            .filter(folders::title.eq(&folder.title))
            .filter(folders::parent_id.eq(&folder.parent_id))
            .first(*&conn)
            .map_err(|e| e.to_string())
    }

    pub fn make(path: &PathBuf, parents: bool, conn: &DbConnection) -> Result<Self, String> {
        let parent = if let Some(path) = path.parent() {
            Self::query(&path.to_path_buf(), parents, *&conn)?
        } else {
            None
        };
        Self::new(
            path.file_name()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or("".to_string()),
            parent,
            *&conn,
        )
    }

    fn query(path: &PathBuf, create: bool, conn: &DbConnection) -> Result<Option<Self>, String> {
        let mut folder: Option<Self> = None;
        for title in path {
            if title.eq("/") {
                // NOTE: "/" only comes in iteration if it absolute path
                // EX: "/test/test" is equivalent to "test/test"
                continue;
            }
            let parent_id = Self::get_id_cow(&folder.as_ref());
            match folders::table
                .filter(folders::title.eq(&title.to_string_lossy()))
                .filter(folders::parent_id.eq(&parent_id))
                .first(*&conn)
            {
                Ok(folder_item) => {
                    folder = Some(folder_item);
                }
                Err(diesel::NotFound) if create => {
                    folder = Some(Self::new(title.to_string_lossy().into(), folder, *&conn)?);
                }
                Err(e) => return Err(e.to_string()),
            }
        }
        Ok(folder)
    }

    fn get_id_cow<'a>(folder: &Option<&'a Self>) -> Cow<'a, str> {
        match folder {
            Some(ref folder_item) => Cow::from(folder_item.id.as_ref().unwrap()),
            None => Cow::from(""),
        }
    }

    pub fn list(path: &PathBuf, conn: &DbConnection) -> Result<(Vec<Folder>, Vec<Note>), String> {
        let folder = Self::query(path, false, *&conn)?;
        Self::list_optional_folder(folder.as_ref(), *&conn)
    }

    fn list_optional_folder(
        folder: Option<&Self>,
        conn: &DbConnection,
    ) -> Result<(Vec<Folder>, Vec<Note>), String> {
        let parent_id = Self::get_id_cow(&folder);
        Ok((
            folders::table
                .filter(folders::parent_id.eq(&parent_id))
                .load::<Self>(*&conn)
                .map_err(|e| e.to_string())?,
            // TODO: handle notes
            vec![],
        ))
    }

    fn delete_folder(&self, recursive: bool, conn: &DbConnection) -> Result<usize, String> {
        let (folders, notes) = Self::list_optional_folder(Some(self), *&conn)?;
        if recursive {
            // TODO: handle notes
            let mut rows = 0;
            for folder in folders {
                rows += folder.delete_folder(recursive, *&conn)?;
            }
            Ok(rows
                + diesel::delete(folders::table.find(&self.id))
                    .execute(*&conn)
                    .map_err(|e| e.to_string())?)
        } else if folders.is_empty() && notes.is_empty() {
            diesel::delete(folders::table.find(&self.id))
                .execute(*&conn)
                .map_err(|e| e.to_string())
        } else {
            return Err("NonEmptyBook".into());
        }
    }

    pub fn delete(path: &PathBuf, recursive: bool, conn: &DbConnection) -> Result<usize, String> {
        let folder = Self::query(path, false, *&conn)?;
        if let Some(folder) = folder {
            conn.transaction::<_, DieselStringError, _>(|| {
                folder
                    .delete_folder(recursive, *&conn)
                    .map_err(|e| DieselStringError(e))
            })
            .map_err(|e| e.0)
        } else {
            Err("NamelessFolder".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::db::{establish_connection, DbConnection};

    use super::Folder;

    #[test]
    pub fn integration_test() {
        let conn = establish_connection().expect("connection or migration failed!");

        {
            let msg = "Initial state check";
            let folder = "";
            assert_folder_count(&folder, 0, &msg, &conn);
        }

        {
            let msg = "list not existing folder";
            let folder = "not_existing";
            assert_folder_not_exists(&folder, &msg, &conn);
        }

        {
            let msg = "Add folder";
            let folder = "test_folder";
            assert_folder_not_exists(&folder, &msg, &conn);
            let execute = Folder::make(&folder.into(), false, &conn);
            assert!(
                execute.is_ok(),
                "{}: {}, hoped for ok of {:#?}",
                msg,
                folder,
                execute
            );
            let new_folder = execute.unwrap();
            assert_eq!(new_folder.title, folder, "{}: {}", msg, folder);
            assert_folder_count("", 1, &msg, &conn);
            assert_folder_count("/", 1, &msg, &conn);
        }

        {
            let msg = "Add nested folder without existing parent and create_parents";
            let folder = "test_folder/test_folder/test_folder";
            let parent_folder = "test_folder/test_folder";
            assert_folder_not_exists(&folder, &msg, &conn);
            assert_folder_not_exists(&parent_folder, &msg, &conn);
            let execute = Folder::make(&folder.into(), false, &conn);
            assert_eq!(
                Folder::make(&folder.into(), false, &conn),
                Err("NotFound".into()),
                "{}: {}, hoped for ok of {:#?}",
                msg,
                folder,
                execute
            );
            assert_folder_not_exists(&folder, &msg, &conn);
            assert_folder_not_exists(&parent_folder, &msg, &conn);
        }
        {
            let msg = "Add nested folder without existing parent but with create_parents";
            let folder = "test_folder/test_folder/grand_child_test_folder";
            let folder_name = "grand_child_test_folder";
            let parent_folder = "test_folder/test_folder";
            assert_folder_not_exists(&folder, &msg, &conn);
            assert_folder_not_exists(&parent_folder, &msg, &conn);

            let execute = Folder::make(&folder.into(), true, &conn);
            assert!(
                execute.is_ok(),
                "{}: {}, hoped for ok of {:#?}",
                msg,
                folder,
                execute
            );
            let new_folder = execute.unwrap();
            assert_eq!(new_folder.title, folder_name, "{}: {}", msg, folder);
            assert_folder_count(parent_folder, 1, &msg, &conn);
        }

        {
            let msg = "Delete nested folder with children but without recursive";
            let parent_folder = "test_folder/test_folder";
            assert_folder_count(parent_folder, 1, &msg, &conn);
            assert_eq!(
                Folder::delete(&parent_folder.into(), false, &conn),
                Err("NonEmptyBook".into()),
                "{}: {}",
                msg,
                parent_folder,
            );
            assert_folder_count(parent_folder, 1, &msg, &conn);
        }
        {
            let msg = "Delete nested folder with children but with recursive";
            let parent_folder = "test_folder/test_folder";
            assert_folder_count(parent_folder, 1, &msg, &conn);
            let execute = Folder::delete(&parent_folder.into(), true, &conn);
            assert!(
                execute.is_ok(),
                "{}: {}, excepted ok of {:#?}",
                msg,
                parent_folder,
                execute
            );
            assert_eq!(execute.unwrap(), 2, "{}: {}", msg, parent_folder);
            assert_folder_not_exists(&parent_folder, &msg, &conn);
        }

        {
            let msg = "Delete empty folder with children without recursive";
            let parent_folder = "test_folder";
            assert_folder_count(parent_folder, 0, &msg, &conn);
            let execute = Folder::delete(&parent_folder.into(), false, &conn);
            assert!(
                execute.is_ok(),
                "{}: {}, excepted ok of {:#?}",
                msg,
                parent_folder,
                execute
            );
            assert_eq!(execute.unwrap(), 1, "{}: {}", msg, parent_folder);
            assert_folder_not_exists(&parent_folder, &msg, &conn);
        }

    }

    fn assert_folder_count(folder: &str, count: usize, msg: &str, conn: &DbConnection) {
        let lists = Folder::list(&folder.into(), &conn);
        assert!(
            lists.is_ok(),
            "{}: {}, hoped for ok of {:#?}",
            msg,
            folder,
            lists
        );
        assert_eq!(
            lists.unwrap().0.len(),
            count,
            "{}: {}, count mismatch",
            msg,
            folder
        );
    }

    fn assert_folder_not_exists(folder: &str, msg: &str, conn: &DbConnection) {
        assert_eq!(
            Folder::list(&folder.into(), &conn),
            Err("NotFound".into()),
            "{}: {}",
            msg,
            folder
        );
    }
}
