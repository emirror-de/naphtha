#![deny(missing_docs)]
//! This library provides several traits in order to make database access a lot
//! easier. In addition when using `naphtha`, it is possible to change
//! the database that is used for specific models in your application without
//! the requirement of changing any code.
//!
//! It implements the most common operations on a database like `insert`, `update`
//! and `remove` for you, while also providing the ability to send custom queries
//! to the database.
//! In addition to that, when using the `barrel-XXX` features, you can write your
//! SQL migrations and use them in your application during runtime.
//! See the [examples](#examples) below.
//!
//! ## Features overview
//!
//! * Most common function implementations `insert`, `update`, `remove` for your
//! models.
//! * Custom transactions provided by the `custom` function
//! * [DatabaseUpdateHandler](DatabaseUpdateHandler) enables you to change the models values before and
//! after the `update` transaction to the database.
//! * Change database on specific model in your application without the need to
//! change your code.
//! * Possibility to query a model from the database by using one of its member.
//! * Integrated [barrel] for writing your SQL migrations and the possibility to apply them during
//! runtime.
//! * Thread safe handling of the database connection.
//!
//! ## Supported databases
//!
//! * SQlite3 (using [diesel](diesel) under the hood).
//! * MySQL (using [diesel](diesel) under the hood).
//! * PostgreSQL (using [diesel](diesel) under the hood).
//!
//! ## Examples
//!
//! In this chapter, minimal usages are shown. Please have a look at the examples
//! in the repository for more and detailed use.
//!
//! ### Connecting to a database
//!
//! ```rust
//! use naphtha::{DatabaseConnection, DatabaseConnect};
//! // This is the only line required to be changed to switch database types.
//! type DbBackend = diesel::SqliteConnection;
//! let db: DatabaseConnection<DbBackend> = DatabaseConnection::connect(":memory:").unwrap();
//! // do some database work
//! ```
//!
//! ### Defining a model and use database connection
//!
//! To create a model and its database integration, the following code is required.
//!
//! *Note that this is an excerpt, see the `examples` folder in the repository for
//! a full working example.*
//!
//! ```ignore
//! #[model(table_name = "persons")]
//! pub struct Person {
//!     id: i32,
//!     pub description: Option<String>,
//!     pub updated_at: NaiveDateTime,
//! }
//!
//! pub mod schema {
//!     table! {
//!         persons (id) {
//!             id -> Int4,
//!             description -> Nullable<Varchar>,
//!             updated_at -> Timestamp,
//!         }
//!     }
//! }
//!
//! impl DatabaseModel for Person {
//!     type PrimaryKey = i32;
//!     fn primary_key(&self) -> Self::PrimaryKey {
//!         self.id
//!     }
//!
//!     fn set_primary_key(&mut self, value: &Self::PrimaryKey) {
//!         self.id = *value;
//!     }
//!
//!     fn default_primary_key() -> Self::PrimaryKey {
//!         0
//!     }
//!
//!     fn table_name() -> &'static str {
//!         "persons"
//!     }
//! }
//!
//! // Define your custom changes to the model before and after the transactions.
//! impl<T> naphtha::DatabaseUpdateHandler<T> for Person {}
//! impl<T> naphtha::DatabaseRemoveHandler<T> for Person {}
//! impl<T> naphtha::DatabaseInsertHandler<T> for Person {}
//!
//! // This implements your database migration functions.
//! impl DatabaseSqlMigration for Person {
//!     fn migration_up(migration: &mut Migration) {
//!         use naphtha::DatabaseModel;
//!         migration.create_table_if_not_exists(Self::table_name(), |t| {
//!             t.add_column("id", types::primary());
//!             t.add_column("description", types::text().nullable(true));
//!             t.add_column("updated_at", types::custom("timestamp"));
//!         });
//!     }
//!
//!     fn migration_down(migration: &mut Migration) {
//!         use naphtha::DatabaseModel;
//!         migration.drop_table_if_exists(Self::table_name());
//!     }
//! }
//!
//! fn main() {
//!     use naphtha::{DatabaseConnection, DatabaseConnect};
//!     let db = DatabaseConnection::connect(":memory:").unwrap();
//!     // p is to be mutable because the insert function updates the id member
//!     // to the one given by the database.
//!     let mut p = Person {
//!         id: Person::default_primary_key(),
//!         description: Some("The new person is registered".into()),
//!     };
//!     p.insert(&db);
//!     // id member is set to the correct number given by the database.
//!
//!     // do a custom query to the database
//!     db.custom::<diesel::result::QueryResult::<Person>, _>(|c: &DbBackend| {
//!         use schema::{persons, persons::dsl::*};
//!         persons.filter(id.eq(1)).first(c)
//!     });
//!
//!     p.remove(&db);
//!     // p not available anymore in the database
//! }
//! ```

pub use diesel;
pub extern crate anyhow;
pub extern crate log;

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// Defines your `struct` as a model and implements the required traits for
/// interacting with the database. Currently only *named* `struct` member are
/// supported.
pub use naphtha_proc_macro::model;

#[cfg(any(
    feature = "barrel-sqlite",
    feature = "barrel-mysql",
    feature = "barrel-pg"
))]
/// Re-exports the [barrel] crate including small trait additions required by naphtha.
pub mod barrel;
mod database_impl;
mod tests;

/// Thin wrapper around a [Connection](diesel::Connection).
pub struct DatabaseConnection<T>(Arc<Mutex<T>>);

impl<T> DatabaseConnection<T> {
    /// Aquires a lock to the wrapped connection.
    pub fn lock(
        &self,
    ) -> Result<MutexGuard<'_, T>, PoisonError<MutexGuard<'_, T>>> {
        self.0.lock()
    }

    /// Executes the custom function to the database instance.
    pub fn custom<R, F>(&self, query: F) -> R
    where
        F: Fn(&T) -> R,
    {
        let c = self.0.lock().expect("Could not aquire connection lock!");
        query(&*c)
    }
}

/// Contains functions database connection handling.
pub trait DatabaseConnect<T> {
    /// Establishes a new connection to the given database string.
    fn connect(database_url: &str) -> anyhow::Result<DatabaseConnection<T>>;
}

/// Defines the relation of the model to the database.
pub trait DatabaseModel {
    /// Defines the primary key type on the database table.
    type PrimaryKey;

    /// Returns the primary key value.
    fn primary_key(&self) -> Self::PrimaryKey;
    /// Sets the primary key value.
    fn set_primary_key(&mut self, value: &Self::PrimaryKey);
    /// Gets the default primary key value.
    fn default_primary_key() -> Self::PrimaryKey;
    /// Returns the table name related to the model.
    fn table_name() -> &'static str;
}

/// Defines functions to modify the stored model instance on the database.
pub trait DatabaseModelModifier<T>
where
    Self: DatabaseUpdateHandler<T>
        + DatabaseInsertHandler<T>
        + DatabaseRemoveHandler<T>,
{
    /// Inserts `self` to the given database.
    /// *Updates the `primary_key` to the one that has been assigned by the database*.
    fn insert(&mut self, conn: &DatabaseConnection<T>) -> anyhow::Result<()>;
    /// Removes `self` from the database, selects by `id`.
    fn remove(&mut self, conn: &DatabaseConnection<T>) -> ::anyhow::Result<()>;
    /// Updates `self` on the given database.
    /// *Updates the `updated_at` member if available before updating the database.*.
    fn update(&mut self, conn: &DatabaseConnection<T>) -> ::anyhow::Result<()>;
}

/// Methods that are called before and after the transaction executed when
/// the [insert](DatabaseModelModifier::insert) method is called.
/// Can be used to do custom changes to the database or the model instance.
/// Useful for extending the basic CRUD model.
#[allow(unused_variables)]
pub trait DatabaseInsertHandler<T> {
    /// This method is called before the transaction to the database takes place.
    fn pre_insert(&mut self, conn: &DatabaseConnection<T>) {}
    /// This method is called after the transaction to the database took place.
    fn post_insert(&mut self, conn: &DatabaseConnection<T>) {}
}

/// Methods that are called before and after the transaction executed when
/// the [update](DatabaseModelModifier::update) method is called.
/// Can be used to do custom changes to the database or the model instance.
/// Useful for extending the basic CRUD model.
#[allow(unused_variables)]
pub trait DatabaseUpdateHandler<T> {
    /// This method is called before the transaction to the database takes place.
    fn pre_update(&mut self, conn: &DatabaseConnection<T>) {}
    /// This method is called after the transaction to the database took place.
    fn post_update(&mut self, conn: &DatabaseConnection<T>) {}
}

/// Methods that are called before and after the transaction executed when
/// the [remove](DatabaseModelModifier::remove) method is called.
/// Can be used to do custom changes to the database or the model instance.
/// Useful for extending the basic CRUD model.
#[allow(unused_variables)]
pub trait DatabaseRemoveHandler<T> {
    /// This method is called before the transaction to the database takes place.
    fn pre_remove(&mut self, conn: &DatabaseConnection<T>) {}
    /// This method is called after the transaction to the database took place.
    fn post_remove(&mut self, conn: &DatabaseConnection<T>) {}
}
