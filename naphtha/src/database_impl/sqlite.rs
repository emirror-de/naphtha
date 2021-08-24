use {
    crate::{DatabaseConnect, DatabaseConnection},
    diesel::{Connection, SqliteConnection},
    std::sync::{Arc, Mutex},
};

impl From<SqliteConnection> for DatabaseConnection<SqliteConnection> {
    fn from(c: SqliteConnection) -> Self {
        DatabaseConnection::<SqliteConnection>(Arc::new(Mutex::new(c)))
    }
}

impl From<Arc<Mutex<SqliteConnection>>>
    for DatabaseConnection<SqliteConnection>
{
    fn from(c: Arc<Mutex<SqliteConnection>>) -> Self {
        DatabaseConnection::<SqliteConnection>(c)
    }
}

impl DatabaseConnect<SqliteConnection>
    for DatabaseConnection<SqliteConnection>
{
    fn connect(
        database_url: &str,
    ) -> Result<DatabaseConnection<SqliteConnection>, String> {
        let connection = match Connection::establish(database_url) {
            Ok(c) => c,
            Err(msg) => {
                return Err(format!(
                "Connection to database \"{}\" could not be established: {}",
                database_url, msg
            ))
            }
        };
        Ok(DatabaseConnection(Arc::new(Mutex::new(connection))))
    }
}
