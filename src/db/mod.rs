pub mod models;
pub mod schema;

use diesel::{connection::Connection, SqliteConnection};

#[cfg(not(test))]
const DATABASE_URL: &str = "target/prod.db";

#[cfg(test)]
const DATABASE_URL: &str = ":memory:";

embed_migrations!();

pub type DbConnection = SqliteConnection;

pub fn establish_connection() -> Result<SqliteConnection, String> {
    let connection = SqliteConnection::establish(&DATABASE_URL).map_err(|e| e.to_string())?;
    embedded_migrations::run(&connection).map_err(|e| e.to_string())?;
    Ok(connection)
}

pub(self) struct DieselStringError(String);

impl From<diesel::result::Error> for DieselStringError {
    fn from(error: diesel::result::Error) -> Self {
        DieselStringError(error.to_string())
    }
}
