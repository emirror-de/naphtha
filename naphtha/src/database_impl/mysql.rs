use {
    crate::{DatabaseConnect, DatabaseConnection},
    diesel::{Connection, MysqlConnection},
    std::sync::{Arc, Mutex},
};

impl From<MysqlConnection> for DatabaseConnection<MysqlConnection> {
    fn from(c: MysqlConnection) -> Self {
        DatabaseConnection::<MysqlConnection>(Arc::new(Mutex::new(c)))
    }
}

impl From<Arc<Mutex<MysqlConnection>>> for DatabaseConnection<MysqlConnection> {
    fn from(c: Arc<Mutex<MysqlConnection>>) -> Self {
        DatabaseConnection::<MysqlConnection>(c)
    }
}

impl DatabaseConnect<MysqlConnection> for DatabaseConnection<MysqlConnection> {
    fn connect(
        database_url: &str,
    ) -> Result<DatabaseConnection<MysqlConnection>, String> {
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
