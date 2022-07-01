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
    ) -> anyhow::Result<DatabaseConnection<PgConnection>> {
        let connection = Connection::establish(database_url)?;
        Ok(DatabaseConnection(Arc::new(Mutex::new(connection))))
    }
}
