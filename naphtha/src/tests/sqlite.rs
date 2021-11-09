#[cfg(test)]
type Database = crate::DatabaseConnection<diesel::SqliteConnection>;

#[test]
fn from_connection() {
    let c: diesel::SqliteConnection =
        diesel::Connection::establish(":memory:").unwrap();
    let _db: Database = crate::DatabaseConnection::from(c);
}

#[test]
fn from_arc_mutex_connection() {
    use std::sync::{Arc, Mutex};
    let c: diesel::SqliteConnection =
        diesel::Connection::establish(":memory:").unwrap();
    let c = Arc::new(Mutex::new(c));
    let _db: Database = crate::DatabaseConnection::from(c);
}

#[test]
fn connect() {
    use crate::{DatabaseConnect, DatabaseConnection};
    let _db: Database = DatabaseConnection::connect(":memory:").unwrap();
}
