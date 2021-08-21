use {
    diesel::{backend::Backend, Connection},
    std::sync::{Arc, Mutex},
};

/// Thin wrapper around a [Connection](diesel::Connection).
pub struct DatabaseConnection<T: diesel::Connection>(Arc<Mutex<T>>);

impl<T: diesel::Connection> DatabaseConnection<T> {
    /// Executes the custom function to the database instance.
    pub fn custom<R>(
        &self,
        query: fn(&T) -> diesel::result::QueryResult<R>,
    ) -> diesel::result::QueryResult<R> {
        let c = self.0.lock().expect("Could not aquire connection lock!");
        query(&*c)
    }
}

impl<T> From<T> for DatabaseConnection<T>
where
    T: diesel::Connection,
{
    fn from(c: T) -> Self {
        DatabaseConnection::<T>(Arc::new(Mutex::new(c)))
    }
}

impl<T> From<Arc<Mutex<T>>> for DatabaseConnection<T>
where
    T: diesel::Connection,
{
    fn from(c: Arc<Mutex<T>>) -> Self {
        DatabaseConnection::<T>(c)
    }
}

/// Defines functions to modify the stored model instance on the database.
pub trait DatabaseModifier<Conn, DB>
where
    Self: Clone,
    Conn: Connection<Backend = DB>,
    DB: Backend + Sized,
{
    /// Inserts `self` to the given database.
    /// *Updates the `id` to the one that has been assigned by the database*.
    fn insert(&mut self, conn: &DatabaseConnection<Conn>) -> bool;
    /// Removes `self` from the database, selects by `id`.
    fn remove(&self, conn: &DatabaseConnection<Conn>) -> bool;
    /// Updates `self` on the given database.
    /// *Updates the `updated_at` member if available before updating the database.*.
    fn update(&mut self, conn: &DatabaseConnection<Conn>) -> bool;
}

#[test]
fn from_connection() {
    let c: diesel::SqliteConnection =
        diesel::Connection::establish(":memory:").unwrap();
    let db = DatabaseConnection::from(c);
}

#[test]
fn from_arc_mutex_connection() {
    let c: diesel::SqliteConnection =
        diesel::Connection::establish(":memory:").unwrap();
    let c = Arc::new(Mutex::new(c));
    let db = DatabaseConnection::from(c);
}
