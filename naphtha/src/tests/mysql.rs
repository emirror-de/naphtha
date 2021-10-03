#[cfg(test)]
type Database = crate::DatabaseConnection<diesel::MysqlConnection>;
#[cfg(test)]
const DATABASE_URL: &'static str = "mysql://naphtha:naphtha@127.0.0.1:3306";

#[test]
fn from_connection() {
    let c: diesel::MysqlConnection =
        diesel::Connection::establish(DATABASE_URL).unwrap();
    let _db: Database = crate::DatabaseConnection::from(c);
}

#[test]
fn from_arc_mutex_connection() {
    use std::sync::{Arc, Mutex};
    let c: diesel::MysqlConnection =
        diesel::Connection::establish(DATABASE_URL).unwrap();
    let c = Arc::new(Mutex::new(c));
    let _db: Database = crate::DatabaseConnection::from(c);
}

#[test]
fn connect() {
    use crate::{DatabaseConnect, DatabaseConnection};
    let _db: Database = DatabaseConnection::connect(DATABASE_URL).unwrap();
}
