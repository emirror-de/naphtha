use {
    crate::{DatabaseConnect, DatabaseConnection},
    diesel::{Connection, PgConnection},
    std::sync::{Arc, Mutex},
};

impl From<PgConnection> for DatabaseConnection<PgConnection> {
    fn from(c: PgConnection) -> Self {
        DatabaseConnection::<PgConnection>(Arc::new(Mutex::new(c)))
    }
}

impl From<Arc<Mutex<PgConnection>>> for DatabaseConnection<PgConnection> {
    fn from(c: Arc<Mutex<PgConnection>>) -> Self {
        DatabaseConnection::<PgConnection>(c)
    }
}

impl DatabaseConnect<PgConnection> for DatabaseConnection<PgConnection> {
    fn connect(
        database_url: &str,
    ) -> Result<DatabaseConnection<PgConnection>, String> {
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
