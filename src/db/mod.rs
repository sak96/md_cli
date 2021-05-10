pub mod models;
pub mod schema;

use diesel::{connection::Connection, SqliteConnection};

embed_migrations!();

pub type DbConnection = SqliteConnection;

pub fn establish_connection() -> Result<SqliteConnection, String> {
    #[cfg(not(test))]
    let db_url = {
        use std::fs::create_dir_all;

        // get local data dir
        let mut path = dirs::data_local_dir().expect("should have local data dir");

        // create local_data_dir/<app>/
        path.push(structopt::clap::crate_name!());
        if !path.exists() {
            create_dir_all(&path).expect(&format!("creating dir {:#?} failed", &path));
        }

        // set path to local_data_dir/<app>/<app.db>
        path.push(format!("{}.db", structopt::clap::crate_name!()));
        path.to_string_lossy().into_owned()
    };

    #[cfg(test)]
    let db_url = ":memory:";

    let connection = SqliteConnection::establish(&db_url).map_err(|e| e.to_string())?;
    embedded_migrations::run(&connection).map_err(|e| e.to_string())?;
    Ok(connection)
}

pub(self) struct DieselStringError(String);

impl From<diesel::result::Error> for DieselStringError {
    fn from(error: diesel::result::Error) -> Self {
        DieselStringError(error.to_string())
    }
}
